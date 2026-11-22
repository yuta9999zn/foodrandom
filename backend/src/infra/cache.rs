use anyhow::{Context, Result};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{AppError, AppResult};

/// Lớp bọc Redis dùng cho: cache phản hồi, khoá phân tán giữa các bản sao và bộ đếm giới hạn tần suất.
///
/// Các thao tác cache (`get_json`, `set_json`) không bao giờ làm hỏng request: Redis lỗi thì
/// ghi log và coi như cache trượt. Thao tác đếm tần suất trả lỗi để tầng trên xử lý.
#[derive(Clone)]
pub struct Cache {
    conn: ConnectionManager,
}

impl Cache {
    pub async fn connect(url: &str) -> Result<Self> {
        let client = redis::Client::open(url).context("REDIS_URL không hợp lệ")?;
        let conn = ConnectionManager::new(client)
            .await
            .context("không kết nối được Redis")?;
        Ok(Self { conn })
    }

    pub async fn ping(&self) -> bool {
        let mut conn = self.conn.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .is_ok()
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.conn.clone();
        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(raw)) => serde_json::from_str(&raw).ok(),
            Ok(None) => None,
            Err(err) => {
                tracing::warn!(%key, error = %err, "đọc cache thất bại");
                None
            }
        }
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) {
        let Ok(raw) = serde_json::to_string(value) else {
            return;
        };
        let mut conn = self.conn.clone();
        if let Err(err) = conn.set_ex::<_, _, ()>(key, raw, ttl_secs.max(1)).await {
            tracing::warn!(%key, error = %err, "ghi cache thất bại");
        }
    }

    /// Đặt khoá nếu chưa tồn tại (dùng làm khoá phân tán giữa các bản sao backend). `true` = lấy được khoá.
    pub async fn try_lock(&self, key: &str, ttl_secs: u64) -> bool {
        let mut conn = self.conn.clone();
        let result: redis::RedisResult<Option<String>> = redis::cmd("SET")
            .arg(key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs.max(1))
            .query_async(&mut conn)
            .await;
        matches!(result, Ok(Some(_)))
    }

    /// Số giây còn lại của khoá; `None` nếu khoá không tồn tại hoặc không có hạn.
    pub async fn ttl(&self, key: &str) -> AppResult<Option<u64>> {
        let mut conn = self.conn.clone();
        let ttl: i64 = conn.ttl(key).await?;
        Ok(if ttl > 0 { Some(ttl as u64) } else { None })
    }

    /// Tăng bộ đếm cửa sổ cố định và trả giá trị sau khi tăng.
    pub async fn hit(&self, key: &str, window_secs: u64) -> AppResult<i64> {
        let mut conn = self.conn.clone();
        let count: i64 = conn.incr(key, 1).await?;
        if count == 1 {
            conn.expire::<_, ()>(key, window_secs as i64).await?;
        }
        Ok(count)
    }

    /// Giới hạn tần suất: quá `limit` lần trong `window_secs` giây thì trả 429.
    pub async fn rate_limit(
        &self,
        key: &str,
        limit: i64,
        window_secs: u64,
        message: &str,
    ) -> AppResult<()> {
        let count = self.hit(key, window_secs).await?;
        if count > limit {
            let retry = self.ttl(key).await.ok().flatten().unwrap_or(window_secs);
            return Err(AppError::too_many(message, retry));
        }
        Ok(())
    }
}
