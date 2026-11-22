use std::env;

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationProviderKind {
    Auto,
    Google,
    Osm,
    Mock,
}

/// Toàn bộ cấu hình của ứng dụng, đọc một lần từ biến môi trường khi khởi động.
#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: String,
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub log_json: bool,

    pub database_url: String,
    pub db_max_connections: u32,
    pub run_migrations: bool,
    pub seed_on_start: bool,
    pub seed_dir: String,

    pub redis_url: String,

    pub cors_origins: Vec<String>,

    pub location_provider: LocationProviderKind,
    pub google_maps_api_key: Option<String>,
    pub overpass_urls: Vec<String>,
    pub nominatim_url: String,
    pub osm_user_agent: String,

    pub max_concurrent_requests: usize,
    pub request_timeout_secs: u64,
    pub food_cache_ttl_secs: u64,
}

fn var(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn var_or(name: &str, default: &str) -> String {
    var(name).unwrap_or_else(|| default.to_string())
}

fn parse<T: std::str::FromStr>(name: &str, default: T) -> Result<T>
where
    T::Err: std::fmt::Display,
{
    match var(name) {
        None => Ok(default),
        Some(raw) => raw
            .parse::<T>()
            .map_err(|e| anyhow::anyhow!("Biến môi trường {name}={raw} không hợp lệ: {e}")),
    }
}

fn parse_bool(name: &str, default: bool) -> bool {
    match var(name).map(|v| v.to_ascii_lowercase()) {
        Some(v) => matches!(v.as_str(), "1" | "true" | "yes" | "on"),
        None => default,
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let app_env = var_or("APP_ENV", "development");

        let location_provider = match var_or("LOCATION_PROVIDER", "auto")
            .to_ascii_lowercase()
            .as_str()
        {
            "auto" => LocationProviderKind::Auto,
            "google" => LocationProviderKind::Google,
            "osm" => LocationProviderKind::Osm,
            "mock" => LocationProviderKind::Mock,
            other => bail!("LOCATION_PROVIDER={other} không hợp lệ (auto | google | osm | mock)"),
        };

        Ok(Self {
            host: var_or("HOST", "0.0.0.0"),
            port: parse("PORT", 8180u16)?,
            log_level: var_or("LOG_LEVEL", "info"),
            log_json: var_or("LOG_FORMAT", "pretty").eq_ignore_ascii_case("json"),

            database_url: var("DATABASE_URL").context("Thiếu biến môi trường DATABASE_URL")?,
            db_max_connections: parse("DB_MAX_CONNECTIONS", 20u32)?,
            run_migrations: parse_bool("RUN_MIGRATIONS", true),
            seed_on_start: parse_bool("SEED_ON_START", true),
            seed_dir: var_or("SEED_DIR", "./seed/foods"),

            redis_url: var("REDIS_URL").context("Thiếu biến môi trường REDIS_URL")?,

            cors_origins: var_or("CORS_ORIGINS", "http://localhost:3100")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),

            location_provider,
            google_maps_api_key: var("GOOGLE_MAPS_API_KEY"),
            overpass_urls: var_or(
                "OVERPASS_URL",
                "https://overpass-api.de/api/interpreter,https://overpass.private.coffee/api/interpreter,https://overpass.kumi.systems/api/interpreter",
            )
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
            nominatim_url: var_or("NOMINATIM_URL", "https://nominatim.openstreetmap.org"),
            osm_user_agent: var_or("OSM_USER_AGENT", "FoodRandom/0.1 (food discovery web app)"),

            max_concurrent_requests: parse("MAX_CONCURRENT_REQUESTS", 2048usize)?,
            request_timeout_secs: parse("REQUEST_TIMEOUT_SECS", 20u64)?,
            food_cache_ttl_secs: parse("FOOD_CACHE_TTL_SECS", 60u64)?,

            app_env,
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }
}
