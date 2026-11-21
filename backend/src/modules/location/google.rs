use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use super::{
    dto::{GeoPoint, RawPlace},
    provider::{NearbySearch, PlaceProvider, CUISINES},
};
use crate::i18n::cuisine_restaurant_query;

const PLACES_TEXT_SEARCH_URL: &str = "https://places.googleapis.com/v1/places:searchText";
const GEOCODE_URL: &str = "https://maps.googleapis.com/maps/api/geocode/json";
const FIELD_MASK: &str = "places.id,places.displayName,places.formattedAddress,places.location,places.rating,places.userRatingCount,places.googleMapsUri,places.currentOpeningHours.openNow";

/// Nhà cung cấp địa điểm dùng Google Maps Platform: Places API (New) — Text Search và Geocoding API.
/// Bật khi có `GOOGLE_MAPS_API_KEY`. Trả kèm điểm đánh giá và số lượt đánh giá của Google.
pub struct GoogleProvider {
    http: reqwest::Client,
    api_key: String,
}

impl GoogleProvider {
    pub fn new(api_key: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .context("không tạo được HTTP client cho Google Maps")?;
        Ok(Self { http, api_key: api_key.to_string() })
    }

    async fn text_search(&self, text: String, cuisine: Option<&str>, search: &NearbySearch) -> Result<Vec<RawPlace>> {
        let body = json!({
            "textQuery": text,
            "languageCode": search.language,
            "regionCode": search.region,
            "pageSize": 20,
            "rankPreference": "DISTANCE",
            "locationBias": {
                "circle": {
                    "center": { "latitude": search.lat, "longitude": search.lng },
                    "radius": f64::from(search.radius_m.min(50_000)),
                }
            }
        });
        let response = self
            .http
            .post(PLACES_TEXT_SEARCH_URL)
            .header("X-Goog-Api-Key", &self.api_key)
            .header("X-Goog-FieldMask", FIELD_MASK)
            .json(&body)
            .send()
            .await
            .context("không gọi được Google Places API")?;
        if !response.status().is_success() {
            let status = response.status();
            let detail = response.text().await.unwrap_or_default();
            bail!("Google Places API trả mã {status}: {}", detail.chars().take(300).collect::<String>());
        }
        let payload: Value = response.json().await.context("phản hồi Google Places không đọc được")?;
        let places = payload.get("places").and_then(Value::as_array).cloned().unwrap_or_default();
        Ok(places
            .iter()
            .filter_map(|p| {
                Some(RawPlace {
                    place_ref: p.get("id")?.as_str()?.to_string(),
                    name: p.get("displayName")?.get("text")?.as_str()?.to_string(),
                    address: p.get("formattedAddress").and_then(Value::as_str).map(str::to_string),
                    lat: p.get("location")?.get("latitude")?.as_f64()?,
                    lng: p.get("location")?.get("longitude")?.as_f64()?,
                    cuisines: cuisine.map(|c| vec![c.to_string()]).unwrap_or_default(),
                    rating: p.get("rating").and_then(Value::as_f64).map(|r| r as f32),
                    rating_count: p.get("userRatingCount").and_then(Value::as_u64).map(|c| c as u32),
                    open_now: p.get("currentOpeningHours").and_then(|h| h.get("openNow")).and_then(Value::as_bool),
                    maps_url: p.get("googleMapsUri").and_then(Value::as_str).map(str::to_string),
                })
            })
            .collect())
    }

    async fn geocode_request(&self, params: &[(&str, String)], language: &str) -> Result<Vec<Value>> {
        let response = self
            .http
            .get(GEOCODE_URL)
            .query(params)
            .query(&[("key", self.api_key.as_str()), ("language", language)])
            .send()
            .await
            .context("không gọi được Google Geocoding API")?;
        let payload: Value = response.json().await.context("phản hồi Google Geocoding không đọc được")?;
        match payload.get("status").and_then(Value::as_str) {
            Some("OK") => Ok(payload.get("results").and_then(Value::as_array).cloned().unwrap_or_default()),
            Some("ZERO_RESULTS") => Ok(vec![]),
            other => bail!("Google Geocoding API trả trạng thái {other:?}"),
        }
    }
}

#[async_trait]
impl PlaceProvider for GoogleProvider {
    fn name(&self) -> &'static str {
        "google"
    }

    fn attribution(&self) -> &'static str {
        "Dữ liệu địa điểm từ Google Maps Platform"
    }

    fn max_radius_m(&self) -> u32 {
        50_000
    }

    async fn nearby(&self, search: &NearbySearch) -> Result<Vec<RawPlace>> {
        let cuisines: Vec<&str> = if search.cuisines.is_empty() {
            CUISINES.to_vec()
        } else {
            search.cuisines.iter().map(String::as_str).collect()
        };

        // Có từ khoá món ăn và đúng một nền ẩm thực: tìm thẳng theo tên món (bằng ngôn ngữ của thị trường).
        if let (Some(keyword), [cuisine]) = (&search.keyword, cuisines.as_slice()) {
            return self.text_search(keyword.clone(), Some(cuisine), search).await;
        }

        let mut all = Vec::new();
        let mut last_error = None;
        let requests = cuisines
            .iter()
            .map(|c| self.text_search(cuisine_restaurant_query(c, &search.language), Some(c), search));
        for result in futures::future::join_all(requests).await {
            match result {
                Ok(places) => all.extend(places),
                Err(err) => last_error = Some(err),
            }
        }
        if all.is_empty() {
            if let Some(err) = last_error {
                return Err(err);
            }
        }
        Ok(all)
    }

    async fn geocode(&self, query: &str, country: &str, language: &str) -> Result<Vec<GeoPoint>> {
        let results = self
            .geocode_request(&[("address", query.to_string()), ("components", format!("country:{country}"))], language)
            .await?;
        Ok(results
            .iter()
            .take(5)
            .filter_map(|r| {
                let location = r.get("geometry")?.get("location")?;
                Some(GeoPoint {
                    label: r.get("formatted_address")?.as_str()?.to_string(),
                    lat: location.get("lat")?.as_f64()?,
                    lng: location.get("lng")?.as_f64()?,
                })
            })
            .collect())
    }

    async fn reverse(&self, lat: f64, lng: f64, language: &str) -> Result<Option<String>> {
        let results = self
            .geocode_request(
                &[
                    ("latlng", format!("{lat:.6},{lng:.6}")),
                    ("result_type", "sublocality|administrative_area_level_2|locality".to_string()),
                ],
                language,
            )
            .await?;
        Ok(results.first().and_then(|r| r.get("formatted_address")).and_then(Value::as_str).map(str::to_string))
    }
}
