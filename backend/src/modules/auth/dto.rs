use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

/// Số điện thoại di động Việt Nam: 0xxxxxxxxx hoặc +84xxxxxxxxx, đầu số 3/5/7/8/9.
pub fn normalize_phone(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| !matches!(c, ' ' | '.' | '-' | '(' | ')')).collect();
    let local = if let Some(rest) = digits.strip_prefix("+84") {
        format!("0{rest}")
    } else if let Some(rest) = digits.strip_prefix("84").filter(|r| r.len() == 9) {
        format!("0{rest}")
    } else {
        digits
    };
    let bytes = local.as_bytes();
    let valid = bytes.len() == 10
        && bytes[0] == b'0'
        && matches!(bytes[1], b'3' | b'5' | b'7' | b'8' | b'9')
        && bytes.iter().all(u8::is_ascii_digit);
    valid.then_some(local)
}

fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    if normalize_phone(phone).is_some() {
        Ok(())
    } else {
        Err(ValidationError::new("phone").with_message("Số điện thoại không hợp lệ (ví dụ 0912345678)".into()))
    }
}

fn validate_otp(code: &str) -> Result<(), ValidationError> {
    if code.len() == 6 && code.bytes().all(|b| b.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::new("otp").with_message("Mã xác thực gồm 6 chữ số".into()))
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterOtpRequest {
    #[validate(email(message = "Email không hợp lệ"), length(max = 254, message = "Email quá dài"))]
    pub email: String,
    #[validate(custom(function = "validate_phone"))]
    pub phone: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VerifyOtpRequest {
    #[validate(email(message = "Email không hợp lệ"))]
    pub email: String,
    #[validate(custom(function = "validate_otp"))]
    pub otp: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Email không hợp lệ"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// Email hoặc số điện thoại đã đăng ký.
    #[validate(length(min = 3, max = 254, message = "Nhập email hoặc số điện thoại"))]
    pub identifier: String,
    #[validate(length(min = 1, max = 128, message = "Nhập mật khẩu"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, max = 128, message = "Nhập mật khẩu hiện tại"))]
    pub current_password: String,
    #[validate(length(min = 8, max = 128, message = "Mật khẩu mới từ 8 đến 128 ký tự"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 60, message = "Tên hiển thị từ 1 đến 60 ký tự"))]
    pub display_name: String,
}

/// Kết quả gửi mã xác thực.
#[derive(Debug, Serialize, ToSchema)]
pub struct OtpSentResponse {
    pub message: String,
    /// Thời gian hiệu lực của mã (giây).
    pub expires_in: u64,
    /// Số giây phải chờ trước khi yêu cầu gửi lại mã.
    pub resend_after: u64,
}

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub phone: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: UserProfile,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRecord {
    #[sqlx(flatten)]
    pub profile: UserProfile,
    pub password_hash: String,
}

#[cfg(test)]
mod tests {
    use super::normalize_phone;

    #[test]
    fn phone_normalization() {
        assert_eq!(normalize_phone("0912345678").as_deref(), Some("0912345678"));
        assert_eq!(normalize_phone("+84 912 345 678").as_deref(), Some("0912345678"));
        assert_eq!(normalize_phone("84912345678").as_deref(), Some("0912345678"));
        assert_eq!(normalize_phone("0212345678"), None);
        assert_eq!(normalize_phone("09123"), None);
        assert_eq!(normalize_phone("09123456789"), None);
    }
}
