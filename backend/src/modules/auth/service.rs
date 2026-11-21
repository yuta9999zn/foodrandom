use std::sync::Arc;

use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    dto::{normalize_phone, OtpSentResponse, UserProfile, UserRecord},
    repository::UserRepository,
    session,
};
use crate::{
    config::Config,
    error::{AppError, AppResult},
    infra::{cache::Cache, mailer::Mailer},
};

const OTP_TTL_SECS: u64 = 10 * 60;
const OTP_RESEND_COOLDOWN_SECS: u64 = 60;
const OTP_MAX_ATTEMPTS: i64 = 5;
const OTP_PER_EMAIL_PER_HOUR: i64 = 5;
const OTP_PER_IP_PER_HOUR: i64 = 20;
const LOGIN_ATTEMPTS_PER_WINDOW: i64 = 10;
const LOGIN_WINDOW_SECS: u64 = 15 * 60;
const PASSWORD_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789";
const GENERATED_PASSWORD_LEN: usize = 12;

#[derive(Debug, Clone, Copy)]
enum OtpPurpose {
    Register,
    ResetPassword,
}

impl OtpPurpose {
    fn key(self) -> &'static str {
        match self {
            OtpPurpose::Register => "register",
            OtpPurpose::ResetPassword => "reset",
        }
    }

    fn label(self) -> &'static str {
        match self {
            OtpPurpose::Register => "đăng ký tài khoản",
            OtpPurpose::ResetPassword => "cấp lại mật khẩu",
        }
    }
}

/// Dữ liệu OTP đang chờ xác thực, lưu trên Redis với hạn 10 phút. Chỉ lưu băm của mã.
#[derive(Debug, Serialize, Deserialize)]
struct PendingOtp {
    code_hash: String,
    phone: Option<String>,
}

/// Đăng ký bằng email + số điện thoại, xác thực OTP qua email, hệ thống cấp mật khẩu và gửi về email.
#[derive(Clone)]
pub struct AuthService {
    users: UserRepository,
    cache: Cache,
    mailer: Mailer,
    cfg: Arc<Config>,
}

impl AuthService {
    pub fn new(users: UserRepository, cache: Cache, mailer: Mailer, cfg: Arc<Config>) -> Self {
        Self { users, cache, mailer, cfg }
    }

    fn otp_hash(&self, email: &str, code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        hasher.update(b":");
        hasher.update(email.as_bytes());
        hasher.update(b":");
        hasher.update(self.cfg.jwt_secret.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn otp_key(purpose: OtpPurpose, email: &str) -> String {
        format!("otp:{}:{email}", purpose.key())
    }

    async fn send_otp(&self, purpose: OtpPurpose, email: &str, phone: Option<String>, ip: &str) -> AppResult<OtpSentResponse> {
        let cooldown_key = format!("otp:cd:{}:{email}", purpose.key());
        if let Some(wait) = self.cache.ttl(&cooldown_key).await? {
            return Err(AppError::too_many(format!("Vui lòng chờ {wait} giây trước khi yêu cầu gửi lại mã"), wait));
        }
        self.cache
            .rate_limit(&format!("otp:hour:email:{email}"), OTP_PER_EMAIL_PER_HOUR, 3600, "Bạn đã yêu cầu quá nhiều mã cho email này, vui lòng thử lại sau")
            .await?;
        self.cache
            .rate_limit(&format!("otp:hour:ip:{ip}"), OTP_PER_IP_PER_HOUR, 3600, "Bạn đã yêu cầu quá nhiều mã, vui lòng thử lại sau")
            .await?;

        let code = format!("{:06}", rand::rng().random_range(0..1_000_000u32));
        let pending = PendingOtp { code_hash: self.otp_hash(email, &code), phone };
        let otp_key = Self::otp_key(purpose, email);
        let attempts_key = format!("{otp_key}:attempts");
        let payload = serde_json::to_string(&pending).map_err(|e| anyhow!(e))?;
        self.cache.set_string(&otp_key, &payload, OTP_TTL_SECS).await?;
        self.cache.delete(&[attempts_key.as_str()]).await?;

        if let Err(err) = self.mailer.send_otp(email, &code, purpose.label(), OTP_TTL_SECS / 60).await {
            tracing::error!(error = ?err, "gửi email OTP thất bại");
            self.cache.delete(&[otp_key.as_str()]).await?;
            return Err(AppError::ServiceUnavailable("Không gửi được email xác thực lúc này, vui lòng thử lại sau".into()));
        }
        self.cache.set_string(&cooldown_key, "1", OTP_RESEND_COOLDOWN_SECS).await?;

        Ok(OtpSentResponse {
            message: format!("Mã xác thực đã được gửi tới {email}"),
            expires_in: OTP_TTL_SECS,
            resend_after: OTP_RESEND_COOLDOWN_SECS,
        })
    }

    /// Kiểm tra mã OTP. Sai quá 5 lần thì huỷ mã. Đúng thì xoá mã (dùng một lần) và trả dữ liệu kèm theo.
    async fn consume_otp(&self, purpose: OtpPurpose, email: &str, code: &str) -> AppResult<PendingOtp> {
        let otp_key = Self::otp_key(purpose, email);
        let attempts_key = format!("{otp_key}:attempts");
        let expired = || AppError::BadRequest("Mã xác thực đã hết hạn hoặc không tồn tại. Hãy yêu cầu mã mới.".into());

        let raw = self.cache.get_string(&otp_key).await?.ok_or_else(expired)?;
        let pending: PendingOtp = serde_json::from_str(&raw).map_err(|_| expired())?;

        let attempts = self.cache.hit(&attempts_key, OTP_TTL_SECS).await?;
        if attempts > OTP_MAX_ATTEMPTS {
            self.cache.delete(&[otp_key.as_str(), attempts_key.as_str()]).await?;
            return Err(AppError::BadRequest("Bạn đã nhập sai quá 5 lần. Hãy yêu cầu mã mới.".into()));
        }
        if !constant_time_eq(pending.code_hash.as_bytes(), self.otp_hash(email, code).as_bytes()) {
            let left = OTP_MAX_ATTEMPTS - attempts;
            return Err(AppError::validation("otp", format!("Mã xác thực không đúng. Còn {left} lần thử.")));
        }
        self.cache.delete(&[otp_key.as_str(), attempts_key.as_str()]).await?;
        Ok(pending)
    }

    pub async fn request_register_otp(&self, email: &str, phone: &str, ip: &str) -> AppResult<OtpSentResponse> {
        let email = normalize_email(email);
        let phone = normalize_phone(phone).ok_or_else(|| AppError::validation("phone", "Số điện thoại không hợp lệ"))?;
        if self.users.email_exists(&email).await? {
            return Err(AppError::Conflict("Email này đã được đăng ký. Hãy đăng nhập hoặc dùng chức năng quên mật khẩu.".into()));
        }
        if self.users.phone_exists(&phone).await? {
            return Err(AppError::Conflict("Số điện thoại này đã được đăng ký.".into()));
        }
        self.send_otp(OtpPurpose::Register, &email, Some(phone), ip).await
    }

    /// Xác thực OTP đăng ký: tạo tài khoản, cấp mật khẩu ngẫu nhiên và gửi mật khẩu về email.
    pub async fn verify_register(&self, email: &str, code: &str) -> AppResult<(UserProfile, bool)> {
        let email = normalize_email(email);
        let pending = self.consume_otp(OtpPurpose::Register, &email, code).await?;
        let phone = pending.phone.ok_or_else(|| AppError::BadRequest("Phiên đăng ký không hợp lệ. Hãy đăng ký lại.".into()))?;

        let password = generate_password();
        let hash = hash_password(password.clone()).await?;
        let display_name = email.split('@').next().unwrap_or("Thành viên").chars().take(60).collect::<String>();

        let user = self.users.create(&email, &phone, &display_name, &hash).await.map_err(|err| match &err {
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                AppError::Conflict("Email hoặc số điện thoại này đã được đăng ký.".into())
            }
            _ => AppError::from(err),
        })?;

        let emailed = match self.mailer.send_password(&email, &password, "Tài khoản FoodRandom của bạn đã được tạo thành công.").await {
            Ok(()) => true,
            Err(err) => {
                tracing::error!(error = ?err, user_id = %user.id, "gửi email mật khẩu thất bại");
                false
            }
        };
        Ok((user, emailed))
    }

    pub async fn request_reset_otp(&self, email: &str, ip: &str) -> AppResult<OtpSentResponse> {
        let email = normalize_email(email);
        // Không tiết lộ email có tồn tại hay không: email lạ vẫn nhận phản hồi giống hệt nhưng không gửi thư.
        if !self.users.email_exists(&email).await? {
            self.cache
                .rate_limit(&format!("otp:hour:ip:{ip}"), OTP_PER_IP_PER_HOUR, 3600, "Bạn đã yêu cầu quá nhiều mã, vui lòng thử lại sau")
                .await?;
            return Ok(OtpSentResponse {
                message: format!("Mã xác thực đã được gửi tới {email}"),
                expires_in: OTP_TTL_SECS,
                resend_after: OTP_RESEND_COOLDOWN_SECS,
            });
        }
        self.send_otp(OtpPurpose::ResetPassword, &email, None, ip).await
    }

    /// Xác thực OTP quên mật khẩu: cấp mật khẩu mới và gửi về email.
    pub async fn verify_reset(&self, email: &str, code: &str) -> AppResult<()> {
        let email = normalize_email(email);
        self.consume_otp(OtpPurpose::ResetPassword, &email, code).await?;
        let user = self
            .users
            .find_by_email(&email)
            .await?
            .ok_or_else(|| AppError::BadRequest("Mã xác thực đã hết hạn hoặc không tồn tại. Hãy yêu cầu mã mới.".into()))?;
        let password = generate_password();
        let hash = hash_password(password.clone()).await?;
        // Gửi thư trước, đổi mật khẩu sau: thư lỗi thì mật khẩu cũ vẫn dùng được.
        self.mailer
            .send_password(&email, &password, "Mật khẩu của bạn đã được cấp lại theo yêu cầu.")
            .await
            .map_err(|err| {
                tracing::error!(error = ?err, "gửi email mật khẩu thất bại");
                AppError::ServiceUnavailable("Không gửi được email lúc này, vui lòng thử lại sau".into())
            })?;
        self.users.set_password(user.profile.id, &hash).await?;
        Ok(())
    }

    pub async fn login(&self, identifier: &str, password: &str, ip: &str) -> AppResult<UserProfile> {
        let identifier = identifier.trim();
        let too_many = "Bạn đăng nhập sai quá nhiều lần, vui lòng thử lại sau 15 phút";
        let id_key = format!("login:fail:id:{}", identifier.to_lowercase());
        let ip_key = format!("login:fail:ip:{ip}");
        for key in [&id_key, &ip_key] {
            let failures: i64 = self.cache.get_string(key).await?.and_then(|v| v.parse().ok()).unwrap_or(0);
            if failures >= LOGIN_ATTEMPTS_PER_WINDOW {
                let wait = self.cache.ttl(key).await?.unwrap_or(LOGIN_WINDOW_SECS);
                return Err(AppError::too_many(too_many, wait));
            }
        }

        let record: Option<UserRecord> = if identifier.contains('@') {
            self.users.find_by_email(&normalize_email(identifier)).await?
        } else {
            match normalize_phone(identifier) {
                Some(phone) => self.users.find_by_phone(&phone).await?,
                None => None,
            }
        };

        let verified = match &record {
            Some(user) => verify_password(password.to_string(), user.password_hash.clone()).await?,
            None => {
                // Vẫn băm một lần để thời gian phản hồi không lộ việc tài khoản có tồn tại hay không.
                let _ = hash_password(password.to_string()).await;
                false
            }
        };

        match (record, verified) {
            (Some(user), true) => {
                self.cache.delete(&[id_key.as_str()]).await?;
                self.users.touch_login(user.profile.id).await?;
                Ok(user.profile)
            }
            _ => {
                self.cache.hit(&id_key, LOGIN_WINDOW_SECS).await?;
                self.cache.hit(&ip_key, LOGIN_WINDOW_SECS).await?;
                Err(AppError::InvalidCredentials("Email/số điện thoại hoặc mật khẩu không đúng".into()))
            }
        }
    }

    pub async fn profile(&self, user_id: Uuid) -> AppResult<UserProfile> {
        Ok(self.users.find_by_id(user_id).await?.ok_or(AppError::Unauthorized)?.profile)
    }

    pub async fn update_profile(&self, user_id: Uuid, display_name: &str) -> AppResult<UserProfile> {
        self.users.update_display_name(user_id, display_name.trim()).await?.ok_or(AppError::Unauthorized)
    }

    pub async fn change_password(&self, user_id: Uuid, current: &str, new_password: &str) -> AppResult<()> {
        let user = self.users.find_by_id(user_id).await?.ok_or(AppError::Unauthorized)?;
        if !verify_password(current.to_string(), user.password_hash).await? {
            return Err(AppError::validation("current_password", "Mật khẩu hiện tại không đúng"));
        }
        let hash = hash_password(new_password.to_string()).await?;
        self.users.set_password(user_id, &hash).await?;
        Ok(())
    }

    pub fn issue_session(&self, user_id: Uuid) -> AppResult<String> {
        Ok(session::issue(user_id, &self.cfg.jwt_secret, self.cfg.session_ttl_hours)?)
    }
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn generate_password() -> String {
    let mut rng = rand::rng();
    (0..GENERATED_PASSWORD_LEN).map(|_| PASSWORD_ALPHABET[rng.random_range(0..PASSWORD_ALPHABET.len())] as char).collect()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Argon2id tốn CPU có chủ đích nên chạy trên luồng blocking để không chặn luồng async.
async fn hash_password(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default().hash_password(password.as_bytes(), &salt).map(|h| h.to_string())
    })
    .await
    .map_err(|e| AppError::Internal(anyhow!(e)))?
    .map_err(|e| AppError::Internal(anyhow!("băm mật khẩu thất bại: {e}")))
}

async fn verify_password(password: String, hash: String) -> AppResult<bool> {
    tokio::task::spawn_blocking(move || match PasswordHash::new(&hash) {
        Ok(parsed) => Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok(),
        Err(_) => false,
    })
    .await
    .map_err(|e| AppError::Internal(anyhow!(e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_password_uses_unambiguous_alphabet() {
        let password = generate_password();
        assert_eq!(password.len(), GENERATED_PASSWORD_LEN);
        assert!(password.bytes().all(|b| PASSWORD_ALPHABET.contains(&b)));
        assert_ne!(password, generate_password());
    }

    #[tokio::test]
    async fn password_hash_roundtrip() {
        let hash = hash_password("Mat-khau-123".into()).await.unwrap();
        assert!(verify_password("Mat-khau-123".into(), hash.clone()).await.unwrap());
        assert!(!verify_password("sai".into(), hash).await.unwrap());
    }
}
