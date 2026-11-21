use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Một địa điểm ăn uống do nhà cung cấp địa điểm trả về.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Place {
    /// Nhà cung cấp dữ liệu: `google`, `osm` hoặc `mock`.
    pub provider: String,
    /// Định danh của địa điểm ở nhà cung cấp.
    pub place_ref: String,
    pub name: String,
    /// Địa chỉ do nhà cung cấp trả về; `null` khi nguồn không có.
    pub address: Option<String>,
    pub lat: f64,
    pub lng: f64,
    /// Khoảng cách đường chim bay từ vị trí người dùng (km).
    pub distance_km: f64,
    /// Các nền ẩm thực của FoodRandom mà địa điểm phục vụ.
    pub cuisines: Vec<String>,
    /// Điểm đánh giá từ nhà cung cấp (chỉ Google có); `null` khi không có dữ liệu.
    pub rating: Option<f32>,
    pub rating_count: Option<u32>,
    /// Đang mở cửa hay không (chỉ Google có); `null` khi không có dữ liệu.
    pub open_now: Option<bool>,
    /// Liên kết mở địa điểm trên Google Maps.
    pub maps_url: String,
    /// `true` nếu đây là dữ liệu GIẢ LẬP của môi trường phát triển.
    pub is_mock: bool,
}

/// Địa điểm thô từ nhà cung cấp, chưa tính khoảng cách. Đây là thứ được cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPlace {
    pub place_ref: String,
    pub name: String,
    pub address: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub cuisines: Vec<String>,
    pub rating: Option<f32>,
    pub rating_count: Option<u32>,
    #[serde(default)]
    pub open_now: Option<bool>,
    pub maps_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeoPoint {
    /// Tên hiển thị của vị trí.
    pub label: String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProviderInfo {
    pub provider: String,
    /// `true` nếu dữ liệu địa điểm là giả lập.
    pub is_mock: bool,
    /// Ghi nguồn dữ liệu cần hiển thị cho người dùng.
    pub attribution: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NearbyResponse {
    #[serde(flatten)]
    pub source: ProviderInfo,
    /// Cách sắp xếp thực tế: `rating` hoặc `distance` (nguồn không có điểm đánh giá thì luôn là `distance`).
    pub sort: String,
    /// Nguồn dữ liệu có điểm đánh giá hay không (Google: có; OpenStreetMap: không).
    pub has_ratings: bool,
    pub places: Vec<Place>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GeocodeResponse {
    #[serde(flatten)]
    pub source: ProviderInfo,
    pub results: Vec<GeoPoint>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReverseResponse {
    #[serde(flatten)]
    pub source: ProviderInfo,
    /// Tên khu vực của toạ độ; `null` nếu nhà cung cấp không xác định được.
    pub label: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct NearbyQuery {
    pub lat: f64,
    pub lng: f64,
    /// Bán kính tìm kiếm (km). Bỏ trống = không giới hạn (tối đa theo nhà cung cấp).
    pub radius_km: Option<f64>,
    /// Lọc theo nền ẩm thực: vietnam, japan, china, korea, uk (cách nhau bởi dấu phẩy).
    pub cuisine: Option<String>,
    /// Slug món ăn: tìm quán phục vụ món này (suy ra nền ẩm thực và từ khoá tên quán).
    pub food: Option<String>,
    /// Số kết quả tối đa (mặc định 12, tối đa 30).
    pub limit: Option<usize>,
    /// `distance` (gần nhất trước) hoặc `rating`: trong các quán gần nhất, quán được đánh giá cao nhất lên trước.
    pub sort: Option<String>,
    /// Thị trường (quốc gia) người dùng đang ở: VN, JP, KR, CN, GB.
    pub market: Option<String>,
    /// Ngôn ngữ kết quả: vi, en, ja, ko, zh.
    pub lang: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GeocodeQuery {
    /// Địa chỉ người dùng nhập tay.
    pub q: String,
    /// Giới hạn kết quả trong quốc gia của thị trường: VN, JP, KR, CN, GB.
    pub market: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ReverseQuery {
    pub lat: f64,
    pub lng: f64,
    pub lang: Option<String>,
}
