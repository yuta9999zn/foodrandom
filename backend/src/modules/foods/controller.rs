use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};

use super::dto::{
    FilterMeta, FoodDetail, FoodListQuery, FoodListResponse, LocaleQuery, RandomRequest,
    RandomResponse,
};
use crate::{
    error::{AppResult, ErrorBody},
    extract::ValidatedJson,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/foods", get(list_foods))
        .route("/foods/{slug}", get(get_food))
        .route("/random", post(random_food))
        .route("/meta/filters", get(filter_meta))
}

/// Danh sách món ăn theo bộ lọc
///
/// Trả 12 món mỗi trang. Khi số món khớp hoàn toàn chưa đủ một trang, phần còn thiếu được lấp bằng
/// gợi ý mở rộng (`match_level = relaxed`). Kèm `lat`/`lng` thì mỗi món có thêm quán gần nhất và khoảng cách.
#[utoipa::path(
    get, path = "/api/v1/foods", tag = "Foods",
    params(FoodListQuery),
    responses(
        (status = 200, body = FoodListResponse),
        (status = 422, body = ErrorBody, description = "Bộ lọc không hợp lệ"),
    )
)]
pub async fn list_foods(
    State(state): State<AppState>,
    Query(query): Query<FoodListQuery>,
) -> AppResult<Json<FoodListResponse>> {
    let (filter, paging) = query.into_parts()?;
    Ok(Json(state.foods.list(filter, paging).await?))
}

/// Chi tiết món ăn
///
/// Mô tả, nguồn gốc, dinh dưỡng (calo, đạm, tinh bột, béo), nguyên liệu, các bước chế biến và món tương tự.
#[utoipa::path(
    get, path = "/api/v1/foods/{slug}", tag = "Foods",
    params(("slug" = String, Path, description = "Slug của món ăn"), LocaleQuery),
    responses((status = 200, body = FoodDetail), (status = 404, body = ErrorBody))
)]
pub async fn get_food(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(locale): Query<LocaleQuery>,
) -> AppResult<Json<FoodDetail>> {
    let (lang, market) = locale.parse()?;
    Ok(Json(state.foods.detail(&slug, lang, market).await?))
}

/// Chọn ngẫu nhiên một món
///
/// Backend dựng lại danh sách từ bộ lọc rồi chọn một món trong 12 món của trang đang xem (`scope = page`)
/// hoặc trong mọi món khớp bộ lọc (`scope = all`). Chọn có trọng số theo `weight`, tránh lặp các món trong `exclude`.
#[utoipa::path(
    post, path = "/api/v1/random", tag = "Random",
    request_body = RandomRequest,
    responses(
        (status = 200, body = RandomResponse),
        (status = 404, body = ErrorBody, description = "Không có món nào để chọn"),
        (status = 422, body = ErrorBody),
    )
)]
pub async fn random_food(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<RandomRequest>,
) -> AppResult<Json<RandomResponse>> {
    Ok(Json(state.foods.random(request).await?))
}

/// Danh mục bộ lọc
#[utoipa::path(get, path = "/api/v1/meta/filters", tag = "Foods", responses((status = 200, body = FilterMeta)))]
pub async fn filter_meta(State(state): State<AppState>) -> AppResult<Json<FilterMeta>> {
    Ok(Json(state.foods.meta().await?))
}
