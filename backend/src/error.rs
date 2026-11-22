use std::collections::BTreeMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use validator::ValidationErrors;

/// Lỗi nghiệp vụ và hạ tầng của toàn ứng dụng. Mọi handler trả `AppResult<T>`;
/// việc chuyển thành phản hồi HTTP được làm tập trung ở `IntoResponse` bên dưới.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("Dữ liệu gửi lên không hợp lệ")]
    Validation(BTreeMap<String, Vec<String>>),
    #[error("{0}")]
    NotFound(String),
    #[error("{message}")]
    TooManyRequests {
        message: String,
        retry_after_secs: u64,
    },
    #[error("{0}")]
    ServiceUnavailable(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;

/// Thân phản hồi lỗi thống nhất của API.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorPayload,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorPayload {
    /// Mã lỗi ổn định cho client xử lý, ví dụ `VALIDATION_ERROR`.
    pub code: String,
    /// Thông báo tiếng Việt hiển thị được cho người dùng.
    pub message: String,
    /// Chi tiết theo trường (chỉ có với lỗi kiểm tra dữ liệu).
    #[schema(value_type = Object)]
    pub details: Option<Value>,
}

impl AppError {
    pub fn validation(field: &str, message: impl Into<String>) -> Self {
        let mut map = BTreeMap::new();
        map.insert(field.to_string(), vec![message.into()]);
        AppError::Validation(map)
    }

    pub fn too_many(message: impl Into<String>, retry_after_secs: u64) -> Self {
        AppError::TooManyRequests {
            message: message.into(),
            retry_after_secs,
        }
    }

    fn parts(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::TooManyRequests { .. } => {
                (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS")
            }
            AppError::ServiceUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE")
            }
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.parts();
        let (message, details) = match &self {
            AppError::Validation(fields) => (self.to_string(), Some(json!(fields))),
            AppError::Internal(err) => {
                // Không lộ chi tiết nội bộ cho client; ghi đầy đủ vào log.
                tracing::error!(error = ?err, "lỗi nội bộ");
                (
                    "Hệ thống đang gặp sự cố, vui lòng thử lại sau".to_string(),
                    None,
                )
            }
            other => (other.to_string(), None),
        };

        let body = ErrorBody {
            error: ErrorPayload {
                code: code.to_string(),
                message,
                details,
            },
        };
        let mut response = (status, Json(body)).into_response();
        if let AppError::TooManyRequests {
            retry_after_secs, ..
        } = self
        {
            if let Ok(value) = retry_after_secs.to_string().parse() {
                response
                    .headers_mut()
                    .insert(axum::http::header::RETRY_AFTER, value);
            }
        }
        response
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Không tìm thấy dữ liệu".into()),
            other => AppError::Internal(anyhow::Error::new(other).context("lỗi cơ sở dữ liệu")),
        }
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::Internal(anyhow::Error::new(err).context("lỗi Redis"))
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        let mut map = BTreeMap::new();
        for (field, list) in errors.field_errors() {
            let messages = list
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("Giá trị không hợp lệ ({})", e.code))
                })
                .collect();
            map.insert(field.to_string(), messages);
        }
        AppError::Validation(map)
    }
}
