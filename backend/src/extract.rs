use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, FromRequest, FromRequestParts, Request},
    http::request::Parts,
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

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
            .map_err(|rejection| {
                AppError::BadRequest(format!(
                    "Dữ liệu JSON không hợp lệ: {}",
                    rejection.body_text()
                ))
            })?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

/// IP của client, ưu tiên header do reverse proxy (Nginx) đặt.
#[derive(Debug, Clone)]
pub struct ClientIp(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = |name: &str| {
            parts
                .headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        };
        let ip = header("x-real-ip")
            .or_else(|| {
                header("x-forwarded-for")
                    .and_then(|v| v.split(',').next().map(|s| s.trim().to_string()))
            })
            .filter(|s| !s.is_empty())
            .or_else(|| {
                parts
                    .extensions
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|c| c.0.ip().to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());
        Ok(ClientIp(ip))
    }
}
