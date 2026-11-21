use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;

use super::{
    dto::{GeoPoint, Place, ProviderInfo, RawPlace},
    google::GoogleProvider,
    mock::MockProvider,
    osm::OsmProvider,
    provider::{NearbySearch, PlaceProvider},
};
use crate::{
    config::{Config, LocationProviderKind},
    error::{AppError, AppResult},
    i18n::Market,
    infra::cache::Cache,
    util::{fold_vi, haversine_km, valid_coords},
};

const NEARBY_CACHE_TTL_SECS: u64 = 15 * 60;
const GEOCODE_CACHE_TTL_SECS: u64 = 24 * 60 * 60;
/// Thời gian chờ tối đa nhà cung cấp địa điểm cho một request người dùng đang đợi (hộp thoại tìm quán).
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(17);
/// Khoá chống nhiều bản sao backend cùng gọi nhà cung cấp cho một ô toạ độ.
const REFRESH_LOCK_SECS: u64 = 45;

/// Tham số tìm quán.
#[derive(Debug, Clone)]
pub struct NearbyParams<'a> {
    pub lat: f64,
    pub lng: f64,
    pub radius_km: Option<f64>,
    pub cuisines: &'a [String],
    pub keyword: Option<&'a str>,
    pub limit: usize,
    pub market: &'a Market,
    pub language: &'a str,
    /// `true`: trong các quán gần nhất, xếp quán được đánh giá cao nhất lên trước.
    pub by_rating: bool,
}

/// Số quán gần nhất được xét khi xếp theo đánh giá: "vài quán quanh đây", không phải quán 5★ ở tận bên kia thành phố.
const RATING_CANDIDATES: usize = 20;
/// Số lượt đánh giá "tin cậy" cho điểm Bayes: quán ít đánh giá bị kéo về mức trung bình.
const RATING_PRIOR_VOTES: f64 = 50.0;

/// Điểm xếp hạng Bayes (kiểu IMDb): (v/(v+m))·R + (m/(v+m))·C. Quán không có điểm xếp sau cùng.
fn weighted_rating(place: &Place, mean: f64) -> f64 {
    match (place.rating, place.rating_count) {
        (Some(r), Some(v)) if v > 0 => {
            let v = f64::from(v);
            (v / (v + RATING_PRIOR_VOTES)) * f64::from(r) + (RATING_PRIOR_VOTES / (v + RATING_PRIOR_VOTES)) * mean
        }
        (Some(r), _) => f64::from(r).min(mean),
        _ => f64::NEG_INFINITY,
    }
}

/// Lấy `RATING_CANDIDATES` quán gần nhất rồi xếp theo điểm Bayes giảm dần (hoà thì quán gần hơn trước).
pub fn rank_by_rating(mut places: Vec<Place>) -> Vec<Place> {
    places.truncate(RATING_CANDIDATES);
    let rated: Vec<f64> = places.iter().filter_map(|p| p.rating.map(f64::from)).collect();
    if rated.is_empty() {
        return places;
    }
    let mean = rated.iter().sum::<f64>() / rated.len() as f64;
    places.sort_by(|a, b| {
        weighted_rating(b, mean).total_cmp(&weighted_rating(a, mean)).then(a.distance_km.total_cmp(&b.distance_km))
    });
    places
}

/// Module Location: tìm quán quanh người dùng ở bất kỳ quốc gia nào, đổi địa chỉ ⇄ toạ độ, tính khoảng cách.
/// Kết quả thô của nhà cung cấp được cache trên Redis theo ô toạ độ ~100 m; khoảng cách luôn
/// được tính lại từ toạ độ thật của người dùng gửi lên.
#[derive(Clone)]
pub struct LocationService {
    provider: Arc<dyn PlaceProvider>,
    cache: Cache,
}

impl LocationService {
    pub fn from_config(cfg: &Config, cache: Cache) -> Result<Self> {
        let osm = || OsmProvider::new(&cfg.overpass_urls, &cfg.nominatim_url, &cfg.osm_user_agent);
        let provider: Arc<dyn PlaceProvider> = match (cfg.location_provider, &cfg.google_maps_api_key) {
            (LocationProviderKind::Mock, _) => {
                if cfg.is_production() {
                    anyhow::bail!("LOCATION_PROVIDER=mock không được dùng ở production");
                }
                tracing::warn!("LOCATION_PROVIDER=mock: dữ liệu địa điểm là GIẢ LẬP");
                Arc::new(MockProvider)
            }
            (LocationProviderKind::Google, Some(key)) | (LocationProviderKind::Auto, Some(key)) => {
                Arc::new(GoogleProvider::new(key)?)
            }
            (LocationProviderKind::Google, None) => {
                anyhow::bail!("LOCATION_PROVIDER=google nhưng thiếu GOOGLE_MAPS_API_KEY")
            }
            (LocationProviderKind::Osm, _) | (LocationProviderKind::Auto, None) => Arc::new(osm()?),
        };
        tracing::info!(provider = provider.name(), "nhà cung cấp địa điểm");
        Ok(Self { provider, cache })
    }

    pub fn info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: self.provider.name().to_string(),
            is_mock: self.provider.is_mock(),
            attribution: self.provider.attribution().to_string(),
        }
    }

    pub fn provider_name(&self) -> &'static str {
        self.provider.name()
    }

    fn check_coords(lat: f64, lng: f64) -> AppResult<()> {
        if valid_coords(lat, lng) {
            Ok(())
        } else {
            Err(AppError::validation("lat", "Toạ độ không hợp lệ"))
        }
    }

    fn radius_m(&self, radius_km: Option<f64>) -> u32 {
        let max_radius = self.provider.max_radius_m();
        radius_km.map(|km| (km * 1000.0).round() as u32).unwrap_or(max_radius).clamp(100, max_radius)
    }

    fn search_for(&self, p: &NearbyParams<'_>) -> (NearbySearch, String) {
        let mut cuisines: Vec<String> = p.cuisines.to_vec();
        cuisines.sort();
        cuisines.dedup();
        let keyword = p.keyword.map(str::trim).filter(|k| !k.is_empty()).map(str::to_string);
        let radius_m = self.radius_m(p.radius_km);
        let key = format!(
            "loc:nearby:{}:{:.3}:{:.3}:{}:{}:{}:{}",
            self.provider.name(),
            p.lat,
            p.lng,
            radius_m,
            cuisines.join(","),
            keyword.as_deref().map(fold_vi).unwrap_or_default(),
            p.language,
        );
        let search = NearbySearch {
            lat: p.lat,
            lng: p.lng,
            radius_m,
            cuisines,
            keyword,
            region: p.market.code.to_string(),
            language: p.language.to_string(),
        };
        (search, key)
    }

    async fn fetch_and_cache(&self, search: &NearbySearch, key: &str) -> AppResult<Vec<RawPlace>> {
        let fetched = tokio::time::timeout(PROVIDER_TIMEOUT, self.provider.nearby(search))
            .await
            .map_err(|_| unavailable("hết thời gian chờ"))?
            .map_err(|err| {
                tracing::warn!(error = ?err, provider = self.provider.name(), "tìm địa điểm thất bại");
                unavailable("nhà cung cấp không phản hồi")
            })?;
        self.cache.set_json(key, &fetched, NEARBY_CACHE_TTL_SECS).await;
        Ok(fetched)
    }

    fn to_places(&self, raw: Vec<RawPlace>, lat: f64, lng: f64, radius_km: Option<f64>, limit: usize) -> Vec<Place> {
        let provider = self.provider.name();
        let is_mock = self.provider.is_mock();
        let limit_km = radius_km.unwrap_or(f64::INFINITY);
        let mut places: Vec<Place> = raw
            .into_iter()
            .map(|p| {
                let distance_km = haversine_km(lat, lng, p.lat, p.lng);
                let maps_url = p
                    .maps_url
                    .unwrap_or_else(|| format!("https://www.google.com/maps/search/?api=1&query={:.6}%2C{:.6}", p.lat, p.lng));
                Place {
                    provider: provider.to_string(),
                    place_ref: p.place_ref,
                    name: p.name,
                    address: p.address,
                    lat: p.lat,
                    lng: p.lng,
                    distance_km: (distance_km * 100.0).round() / 100.0,
                    cuisines: p.cuisines,
                    rating: p.rating,
                    rating_count: p.rating_count,
                    open_now: p.open_now,
                    maps_url,
                    is_mock,
                }
            })
            .filter(|p| p.distance_km <= limit_km)
            .collect();
        places.sort_by(|a, b| a.distance_km.total_cmp(&b.distance_km));
        places.truncate(limit);
        places
    }

    /// Tìm quán quanh toạ độ, sắp theo khoảng cách tăng dần. Người dùng đang chờ nên gọi thẳng nhà cung cấp nếu cache trượt.
    pub async fn nearby(&self, p: NearbyParams<'_>) -> AppResult<Vec<Place>> {
        Self::check_coords(p.lat, p.lng)?;
        let (search, key) = self.search_for(&p);
        let raw = match self.cache.get_json::<Vec<RawPlace>>(&key).await {
            Some(hit) => hit,
            None => self.fetch_and_cache(&search, &key).await?,
        };
        if p.by_rating && self.has_ratings() {
            let mut places = rank_by_rating(self.to_places(raw, p.lat, p.lng, p.radius_km, usize::MAX));
            places.truncate(p.limit);
            Ok(places)
        } else {
            Ok(self.to_places(raw, p.lat, p.lng, p.radius_km, p.limit))
        }
    }

    /// Nhà cung cấp hiện tại có trả điểm đánh giá hay không (Google: có; OpenStreetMap / giả lập: không).
    pub fn has_ratings(&self) -> bool {
        self.provider.name() == "google"
    }

    /// Quán gần nhất của từng nền ẩm thực, chỉ lấy từ cache. Cache trượt thì khởi động việc tìm ở nền
    /// (mỗi ô toạ độ chỉ một bản sao backend làm nhờ khoá Redis) và trả `None` ngay — danh sách món không phải đợi.
    pub async fn nearest_by_cuisine_cached(
        &self,
        lat: f64,
        lng: f64,
        radius_km: Option<f64>,
        market: &'static Market,
    ) -> AppResult<Option<HashMap<String, Place>>> {
        Self::check_coords(lat, lng)?;
        let params = NearbyParams {
            lat,
            lng,
            radius_km,
            cuisines: &[],
            keyword: None,
            limit: usize::MAX,
            market,
            language: market.language,
            by_rating: false,
        };
        let (search, key) = self.search_for(&params);

        if let Some(raw) = self.cache.get_json::<Vec<RawPlace>>(&key).await {
            let places = self.to_places(raw, lat, lng, radius_km, usize::MAX);
            let mut nearest: HashMap<String, Place> = HashMap::new();
            // `places` đã sắp theo khoảng cách nên lần gặp đầu tiên của mỗi nền ẩm thực là gần nhất.
            for place in places {
                for cuisine in place.cuisines.clone() {
                    nearest.entry(cuisine).or_insert_with(|| place.clone());
                }
            }
            return Ok(Some(nearest));
        }

        if self.cache.try_lock(&format!("{key}:lock"), REFRESH_LOCK_SECS).await {
            let this = self.clone();
            tokio::spawn(async move {
                if let Err(err) = this.fetch_and_cache(&search, &key).await {
                    tracing::warn!(error = %err, "tìm quán ở nền thất bại");
                }
            });
        }
        Ok(None)
    }

    pub async fn geocode(&self, query: &str, market: &Market, language: &str) -> AppResult<Vec<GeoPoint>> {
        let query = query.trim();
        if query.chars().count() < 2 {
            return Err(AppError::validation("q", "Nhập tối thiểu 2 ký tự địa chỉ"));
        }
        if query.chars().count() > 200 {
            return Err(AppError::validation("q", "Địa chỉ quá dài"));
        }
        let cache_key = format!("loc:geocode:{}:{}:{}:{}", self.provider.name(), market.code, language, fold_vi(query));
        if let Some(hit) = self.cache.get_json::<Vec<GeoPoint>>(&cache_key).await {
            return Ok(hit);
        }
        let results = tokio::time::timeout(PROVIDER_TIMEOUT, self.provider.geocode(query, market.code, language))
            .await
            .map_err(|_| unavailable("hết thời gian chờ"))?
            .map_err(|err| {
                tracing::warn!(error = ?err, "geocode thất bại");
                unavailable("nhà cung cấp không phản hồi")
            })?;
        self.cache.set_json(&cache_key, &results, GEOCODE_CACHE_TTL_SECS).await;
        Ok(results)
    }

    pub async fn reverse(&self, lat: f64, lng: f64, language: &str) -> AppResult<Option<String>> {
        Self::check_coords(lat, lng)?;
        let cache_key = format!("loc:reverse:{}:{}:{:.3}:{:.3}", self.provider.name(), language, lat, lng);
        if let Some(hit) = self.cache.get_json::<Option<String>>(&cache_key).await {
            return Ok(hit);
        }
        let label = tokio::time::timeout(PROVIDER_TIMEOUT, self.provider.reverse(lat, lng, language))
            .await
            .map_err(|_| unavailable("hết thời gian chờ"))?
            .map_err(|err| {
                tracing::warn!(error = ?err, "reverse geocode thất bại");
                unavailable("nhà cung cấp không phản hồi")
            })?;
        self.cache.set_json(&cache_key, &label, GEOCODE_CACHE_TTL_SECS).await;
        Ok(label)
    }
}

fn unavailable(reason: &str) -> AppError {
    AppError::ServiceUnavailable(format!("Dịch vụ địa điểm tạm thời không dùng được ({reason}). Vui lòng thử lại sau."))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(name: &str, distance_km: f64, rating: Option<f32>, count: Option<u32>) -> Place {
        Place {
            provider: "google".into(),
            place_ref: name.into(),
            name: name.into(),
            address: None,
            lat: 0.0,
            lng: 0.0,
            distance_km,
            cuisines: vec![],
            rating,
            rating_count: count,
            open_now: None,
            maps_url: String::new(),
            is_mock: false,
        }
    }

    #[test]
    fn well_reviewed_beats_few_perfect_reviews() {
        let ranked = rank_by_rating(vec![
            place("gan-nhat", 0.2, Some(4.0), Some(300)),
            place("5-sao-2-danh-gia", 0.5, Some(5.0), Some(2)),
            place("4.7-sao-2000-danh-gia", 0.9, Some(4.7), Some(2000)),
            place("khong-co-diem", 0.1, None, None),
        ]);
        let order: Vec<&str> = ranked.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(order[0], "4.7-sao-2000-danh-gia");
        assert_eq!(*order.last().unwrap(), "khong-co-diem");
    }

    #[test]
    fn only_nearby_candidates_are_ranked() {
        let mut places: Vec<Place> = (0..25).map(|i| place(&format!("p{i}"), f64::from(i), Some(4.0), Some(100))).collect();
        places.push(place("xa-nhung-5-sao", 50.0, Some(5.0), Some(5000)));
        let ranked = rank_by_rating(places);
        assert_eq!(ranked.len(), 20);
        assert!(ranked.iter().all(|p| p.name != "xa-nhung-5-sao"));
    }
}
