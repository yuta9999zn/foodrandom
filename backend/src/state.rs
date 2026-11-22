use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;

use crate::{
    config::Config,
    infra::cache::Cache,
    modules::{
        foods::{repository::FoodRepository, service::FoodService},
        location::service::LocationService,
    },
};

/// Trạng thái dùng chung của ứng dụng. Mọi thành phần đều `Clone` rẻ (bọc `Arc` hoặc pool),
/// không giữ trạng thái theo người dùng trong bộ nhớ nên chạy được nhiều bản sao sau cân bằng tải.
#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub db: PgPool,
    pub cache: Cache,
    pub foods: FoodService,
    pub location: LocationService,
}

impl AppState {
    pub fn build(cfg: Arc<Config>, db: PgPool, cache: Cache) -> Result<Self> {
        let location = LocationService::from_config(&cfg, cache.clone())?;
        let foods = FoodService::new(
            FoodRepository::new(db.clone()),
            cache.clone(),
            location.clone(),
            cfg.food_cache_ttl_secs,
        );
        Ok(Self {
            cfg,
            db,
            cache,
            foods,
            location,
        })
    }
}
