use anyhow::Result;
use async_trait::async_trait;

use super::dto::{GeoPoint, RawPlace};

/// Yêu cầu tìm địa điểm quanh một toạ độ.
#[derive(Debug, Clone)]
pub struct NearbySearch {
    pub lat: f64,
    pub lng: f64,
    pub radius_m: u32,
    /// Mã nền ẩm thực của FoodRandom; rỗng = cả năm nền.
    pub cuisines: Vec<String>,
    /// Từ khoá tên món / tên quán (tuỳ chọn), viết bằng ngôn ngữ của thị trường.
    pub keyword: Option<String>,
    /// Mã quốc gia ISO của thị trường (VN, JP, …): ưu tiên kết quả trong nước đó.
    pub region: String,
    /// Ngôn ngữ trả kết quả (tên quán, địa chỉ).
    pub language: String,
}

/// Cổng trừu tượng tới nhà cung cấp địa điểm. Module Location chỉ phụ thuộc vào trait này,
/// nên đổi Google ⇄ OpenStreetMap ⇄ giả lập không ảnh hưởng phần còn lại của hệ thống.
/// Mọi nhà cung cấp đều hoạt động trên toàn thế giới: người ở Nhật tìm món Việt sẽ thấy quán Việt ở Nhật.
#[async_trait]
pub trait PlaceProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn attribution(&self) -> &'static str;
    fn is_mock(&self) -> bool {
        false
    }
    /// Bán kính lớn nhất (m) nhà cung cấp chấp nhận; dùng khi người dùng chọn "Không giới hạn".
    fn max_radius_m(&self) -> u32;

    async fn nearby(&self, search: &NearbySearch) -> Result<Vec<RawPlace>>;
    /// Địa chỉ nhập tay → toạ độ, giới hạn trong quốc gia `country` (ISO alpha-2).
    async fn geocode(&self, query: &str, country: &str, language: &str) -> Result<Vec<GeoPoint>>;
    async fn reverse(&self, lat: f64, lng: f64, language: &str) -> Result<Option<String>>;
}

pub const CUISINES: [&str; 5] = ["vietnam", "japan", "china", "korea", "uk"];

pub fn cuisine_label_vi(code: &str) -> &'static str {
    match code {
        "vietnam" => "Việt Nam",
        "japan" => "Nhật Bản",
        "china" => "Trung Quốc",
        "korea" => "Hàn Quốc",
        "uk" => "Anh",
        _ => "",
    }
}
