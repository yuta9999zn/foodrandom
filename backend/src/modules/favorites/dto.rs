use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::modules::foods::dto::FoodSummary;

#[derive(Debug, Serialize, ToSchema)]
pub struct FavoriteFoodsResponse {
    pub items: Vec<FoodSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FavoriteFoodSlugs {
    pub slugs: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FavoriteToggleResponse {
    pub slug: String,
    pub favorited: bool,
}

/// Quán ăn người dùng đã lưu.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct FavoritePlace {
    pub id: Uuid,
    /// Nguồn dữ liệu của quán: google | osm | mock.
    pub provider: String,
    pub place_ref: String,
    pub name: String,
    pub address: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub cuisine: Option<String>,
    /// Món ăn gắn với quán lúc lưu (nếu lưu từ trang chi tiết món).
    pub food_slug: Option<String>,
    pub food_name: Option<String>,
    pub maps_url: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FavoritePlacesResponse {
    pub items: Vec<FavoritePlace>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SavePlaceRequest {
    #[validate(length(min = 1, max = 20, message = "Thiếu nhà cung cấp"))]
    pub provider: String,
    #[validate(length(min = 1, max = 200, message = "Thiếu định danh địa điểm"))]
    pub place_ref: String,
    #[validate(length(min = 1, max = 200, message = "Tên quán từ 1 đến 200 ký tự"))]
    pub name: String,
    #[validate(length(max = 400, message = "Địa chỉ tối đa 400 ký tự"))]
    pub address: Option<String>,
    #[validate(range(min = -90.0, max = 90.0, message = "Vĩ độ không hợp lệ"))]
    pub lat: f64,
    #[validate(range(min = -180.0, max = 180.0, message = "Kinh độ không hợp lệ"))]
    pub lng: f64,
    #[validate(length(max = 20))]
    pub cuisine: Option<String>,
    #[validate(length(max = 120))]
    pub food_slug: Option<String>,
    #[validate(length(max = 500), url(message = "Liên kết bản đồ không hợp lệ"))]
    pub maps_url: Option<String>,
    #[validate(length(max = 300, message = "Ghi chú tối đa 300 ký tự"))]
    pub note: Option<String>,
}
