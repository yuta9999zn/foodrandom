use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, FromRequest, FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts},
    Json,
};
use axum_extra::extract::CookieJar;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use validator::Validate;

use crate::{error::AppError, modules::auth::session, state::AppState};

/// JSON body đã qua kiểm tra `validator`. Sai định dạng trả 400, sai ràng buộc trả 422 kèm chi tiết theo trường.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| AppError::BadRequest(format!("Dữ liệu JSON không hợp lệ: {}", rejection.body_text())))?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

/// Người dùng đã đăng nhập. Phiên lấy từ cookie `fr_session` hoặc header `Authorization: Bearer`.
#[derive(Debug, Clone, Copy)]
pub struct AuthUser(pub Uuid);

fn session_token(parts: &Parts) -> Option<String> {
    let jar = CookieJar::from_headers(&parts.headers);
    if let Some(cookie) = jar.get(session::COOKIE_NAME) {
        return Some(cookie.value().to_string());
    }
    parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v.trim().to_string())
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = session_token(parts).ok_or(AppError::Unauthorized)?;
        let user_id = session::verify(&token, &state.cfg.jwt_secret).ok_or(AppError::Unauthorized)?;
        Ok(AuthUser(user_id))
    }
}

/// Người dùng nếu có đăng nhập; khách thì `None`.
#[derive(Debug, Clone, Copy)]
pub struct MaybeAuthUser(pub Option<Uuid>);

impl FromRequestParts<AppState> for MaybeAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = session_token(parts).and_then(|t| session::verify(&t, &state.cfg.jwt_secret));
        Ok(MaybeAuthUser(user))
    }
}

/// IP của client, ưu tiên header do reverse proxy (Nginx) đặt.
#[derive(Debug, Clone)]
pub struct ClientIp(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = |name: &str| parts.headers.get(name).and_then(|v| v.to_str().ok()).map(str::to_string);
        let ip = header("x-real-ip")
            .or_else(|| header("x-forwarded-for").and_then(|v| v.split(',').next().map(|s| s.trim().to_string())))
            .filter(|s| !s.is_empty())
            .or_else(|| parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|c| c.0.ip().to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        Ok(ClientIp(ip))
    }
}
