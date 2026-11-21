use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const COOKIE_NAME: &str = "fr_session";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    iat: i64,
    exp: i64,
}

/// Phát hành JWT (HS256) cho phiên đăng nhập. Phiên không lưu trạng thái ở máy chủ,
/// nên mọi bản sao backend sau cân bằng tải đều xác thực được.
pub fn issue(user_id: Uuid, secret: &str, ttl_hours: i64) -> anyhow::Result<String> {
    let now = Utc::now().timestamp();
    let claims = Claims { sub: user_id, iat: now, exp: now + ttl_hours * 3600 };
    Ok(encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes()))?)
}

pub fn verify(token: &str, secret: &str) -> Option<Uuid> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation).ok().map(|data| data.claims.sub)
}

pub fn cookie(token: String, ttl_hours: i64, secure: bool) -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(time_duration(ttl_hours * 3600))
        .build()
}

pub fn clear_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, "")).path("/").http_only(true).same_site(SameSite::Lax).secure(secure).max_age(time_duration(0)).build()
}

fn time_duration(seconds: i64) -> time::Duration {
    time::Duration::seconds(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_roundtrip_and_tamper_detection() {
        let user = Uuid::new_v4();
        let token = issue(user, "secret-one", 1).unwrap();
        assert_eq!(verify(&token, "secret-one"), Some(user));
        assert_eq!(verify(&token, "secret-two"), None);
        assert_eq!(verify("not-a-token", "secret-one"), None);
    }

    #[test]
    fn expired_token_is_rejected() {
        let token = issue(Uuid::new_v4(), "secret", -2).unwrap();
        assert_eq!(verify(&token, "secret"), None);
    }
}
