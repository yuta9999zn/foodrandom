use anyhow::Result;
use async_trait::async_trait;

use super::{
    dto::{GeoPoint, RawPlace},
    provider::{cuisine_label_vi, NearbySearch, PlaceProvider, CUISINES},
};

/// Nhà cung cấp GIẢ LẬP, chỉ bật khi `LOCATION_PROVIDER=mock`.
///
/// Sinh các quán không có thật quanh toạ độ được gửi lên để phát triển giao diện khi không có mạng.
/// Mọi địa điểm đều mang `is_mock = true`, tên bắt đầu bằng "[GIẢ LẬP]" và giao diện hiển thị nhãn cảnh báo.
/// Không dùng ở production.
pub struct MockProvider;

/// (khoảng cách km, góc phương vị độ) của các quán giả lập so với người dùng.
const OFFSETS: [(f64, f64); 3] = [(0.4, 35.0), (1.1, 160.0), (2.6, 285.0)];

fn offset(lat: f64, lng: f64, distance_km: f64, bearing_deg: f64) -> (f64, f64) {
    let bearing = bearing_deg.to_radians();
    let d_lat = distance_km / 111.32 * bearing.cos();
    let d_lng = distance_km / (111.32 * lat.to_radians().cos().max(0.01)) * bearing.sin();
    (lat + d_lat, lng + d_lng)
}

#[async_trait]
impl PlaceProvider for MockProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn attribution(&self) -> &'static str {
        "DỮ LIỆU GIẢ LẬP — các quán dưới đây không có thật, chỉ dùng cho môi trường phát triển"
    }

    fn is_mock(&self) -> bool {
        true
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
        let mut places = Vec::new();
        for (ci, cuisine) in cuisines.iter().enumerate() {
            for (i, (distance, bearing)) in OFFSETS.iter().enumerate() {
                let (lat, lng) = offset(
                    search.lat,
                    search.lng,
                    distance + ci as f64 * 0.15,
                    bearing + ci as f64 * 23.0,
                );
                places.push(RawPlace {
                    place_ref: format!("mock-{cuisine}-{}", i + 1),
                    name: format!("[GIẢ LẬP] Quán {} mẫu {}", cuisine_label_vi(cuisine), i + 1),
                    address: Some("Địa chỉ giả lập — không có thật".to_string()),
                    lat,
                    lng,
                    cuisines: vec![cuisine.to_string()],
                    rating: None,
                    rating_count: None,
                    open_now: None,
                    maps_url: None,
                });
            }
        }
        Ok(places)
    }

    async fn geocode(&self, query: &str, country: &str, _language: &str) -> Result<Vec<GeoPoint>> {
        // Toạ độ trung tâm thành phố lớn của thị trường — cố định cho mọi truy vấn giả lập.
        let market = crate::i18n::market(country).unwrap_or(&crate::i18n::MARKETS[0]);
        Ok(vec![GeoPoint {
            label: format!("[GIẢ LẬP] {query} → trung tâm {}", market.code),
            lat: market.center_lat,
            lng: market.center_lng,
        }])
    }

    async fn reverse(&self, _lat: f64, _lng: f64, _language: &str) -> Result<Option<String>> {
        Ok(Some("[GIẢ LẬP] Vị trí của bạn".to_string()))
    }
}
