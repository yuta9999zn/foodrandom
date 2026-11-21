use sqlx::PgPool;
use uuid::Uuid;

use super::dto::{UserProfile, UserRecord};

const PROFILE_COLUMNS: &str = "id, email, phone, display_name, created_at";

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn email_exists(&self, email: &str) -> sqlx::Result<bool> {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE lower(email) = lower($1))").bind(email).fetch_one(&self.pool).await
    }

    pub async fn phone_exists(&self, phone: &str) -> sqlx::Result<bool> {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE phone = $1)").bind(phone).fetch_one(&self.pool).await
    }

    pub async fn create(&self, email: &str, phone: &str, display_name: &str, password_hash: &str) -> sqlx::Result<UserProfile> {
        sqlx::query_as::<_, UserProfile>(&format!(
            "INSERT INTO users (email, phone, display_name, password_hash, last_login_at) \
             VALUES ($1, $2, $3, $4, now()) RETURNING {PROFILE_COLUMNS}"
        ))
        .bind(email)
        .bind(phone)
        .bind(display_name)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_email(&self, email: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as::<_, UserRecord>(&format!("SELECT {PROFILE_COLUMNS}, password_hash FROM users WHERE lower(email) = lower($1)"))
            .bind(email)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by_phone(&self, phone: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as::<_, UserRecord>(&format!("SELECT {PROFILE_COLUMNS}, password_hash FROM users WHERE phone = $1"))
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as::<_, UserRecord>(&format!("SELECT {PROFILE_COLUMNS}, password_hash FROM users WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn set_password(&self, id: Uuid, password_hash: &str) -> sqlx::Result<()> {
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2").bind(password_hash).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn touch_login(&self, id: Uuid) -> sqlx::Result<()> {
        sqlx::query("UPDATE users SET last_login_at = now() WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn update_display_name(&self, id: Uuid, display_name: &str) -> sqlx::Result<Option<UserProfile>> {
        sqlx::query_as::<_, UserProfile>(&format!("UPDATE users SET display_name = $1 WHERE id = $2 RETURNING {PROFILE_COLUMNS}"))
            .bind(display_name)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
}
