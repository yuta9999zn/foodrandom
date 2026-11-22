mod config;
mod error;
mod extract;
mod i18n;
mod infra;
mod modules;
mod openapi;
mod state;
mod util;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::{
    error_handling::HandleErrorLayer,
    extract::State,
    http::{header, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::get,
    BoxError, Json, Router,
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{config::Config, error::AppError, infra::cache::Cache, state::AppState};

#[tokio::main]
async fn main() -> Result<()> {
    // .env chỉ để tiện chạy cục bộ; ở container cấu hình đến từ biến môi trường.
    let _ = dotenvy::dotenv();
    let cfg = Arc::new(Config::from_env()?);
    init_tracing(&cfg);

    let db = PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect(&cfg.database_url)
        .await
        .context("không kết nối được PostgreSQL")?;

    if cfg.run_migrations {
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .context("chạy migration thất bại")?;
        tracing::info!("migration đã áp dụng");
    }
    if cfg.seed_on_start {
        let count = infra::seed::seed_foods(&db, &cfg.seed_dir)
            .await
            .context("nạp dữ liệu seed thất bại")?;
        tracing::info!(count, "đã nạp dữ liệu món ăn từ seed");
    }

    let cache = Cache::connect(&cfg.redis_url).await?;
    let state = AppState::build(cfg.clone(), db, cache)?;
    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port)
        .parse()
        .context("HOST/PORT không hợp lệ")?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("không mở được cổng {addr}"))?;
    tracing::info!(%addr, env = %cfg.app_env, "FoodRandom API đang chạy — tài liệu API ở /docs");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("máy chủ dừng do lỗi")?;
    tracing::info!("đã dừng máy chủ");
    Ok(())
}

fn init_tracing(cfg: &Config) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{},tower_http=info,sqlx=warn", cfg.log_level)));
    let registry = tracing_subscriber::registry().with(filter);
    if cfg.log_json {
        registry
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        registry
            .with(tracing_subscriber::fmt::layer().compact())
            .init();
    }
}

fn build_router(state: AppState) -> Router {
    let cfg = state.cfg.clone();

    let origins: Vec<HeaderValue> = cfg
        .cors_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
        .max_age(Duration::from_secs(600));

    let api = Router::new()
        .merge(modules::foods::controller::router())
        .merge(modules::location::controller::router());

    // Thứ tự từ ngoài vào trong. Chống quá tải: quá số request đồng thời cho phép thì trả 503 ngay
    // (load shed) thay vì xếp hàng vô hạn; request chạy quá lâu bị cắt bằng timeout.
    let middleware = ServiceBuilder::new()
        .layer(CatchPanicLayer::new())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(HandleErrorLayer::new(handle_middleware_error))
        .load_shed()
        .concurrency_limit(cfg.max_concurrent_requests)
        .timeout(Duration::from_secs(cfg.request_timeout_secs))
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(cors);

    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api-docs/openapi.json", get(openapi::openapi_json))
        .route("/docs", get(openapi::swagger_ui))
        .nest("/api/v1", api)
        .fallback(|| async { AppError::NotFound("Không tìm thấy đường dẫn".into()) })
        .layer(middleware)
        .with_state(state)
}

async fn handle_middleware_error(err: BoxError) -> AppError {
    if err.is::<tower::timeout::error::Elapsed>() {
        AppError::ServiceUnavailable("Yêu cầu xử lý quá lâu, vui lòng thử lại".into())
    } else if err.is::<tower::load_shed::error::Overloaded>() {
        AppError::ServiceUnavailable("Hệ thống đang quá tải, vui lòng thử lại sau giây lát".into())
    } else {
        AppError::Internal(anyhow::anyhow!("lỗi middleware: {err}"))
    }
}

/// Liveness: tiến trình còn sống.
async fn health() -> Json<serde_json::Value> {
    Json(
        json!({ "status": "ok", "service": "foodrandom-api", "version": env!("CARGO_PKG_VERSION") }),
    )
}

/// Readiness: PostgreSQL và Redis đều trả lời. Cân bằng tải dùng điểm này để quyết định có chuyển request tới không.
async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();
    let redis_ok = state.cache.ping().await;
    let status = if db_ok && redis_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(
            json!({ "status": if db_ok && redis_ok { "ready" } else { "degraded" }, "postgres": db_ok, "redis": redis_ok }),
        ),
    )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("nhận tín hiệu dừng, đang đóng kết nối…");
}
