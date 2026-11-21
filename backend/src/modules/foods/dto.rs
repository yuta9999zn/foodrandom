use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, AppResult},
    i18n::{self, BUDGET_CODES},
    modules::location::dto::{Place, ProviderInfo},
    util::valid_coords,
};

pub const PAGE_SIZE: usize = 12;
pub const MAX_PAGE_SIZE: usize = 48;

pub const CUISINES: [&str; 5] = ["vietnam", "japan", "china", "korea", "uk"];
pub const STYLES: [&str; 2] = ["traditional", "modern"];
pub const DISH_TYPES: [&str; 6] = ["main", "side", "fast_food", "snack", "dessert", "drink"];
pub const MEALS: [&str; 4] = ["breakfast", "lunch", "dinner", "snack"];
pub const SOUP_TYPES: [&str; 8] = ["pho", "bun", "mi", "hu_tieu", "banh_canh", "lau", "kho", "khac"];
pub const SORTS: [&str; 6] = ["featured", "nearest", "price_asc", "price_desc", "rating", "popular"];

/// Món ăn ở dạng tóm tắt, dùng cho thẻ món trong danh sách.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct FoodSummary {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub original_name: Option<String>,
    /// vietnam | japan | china | korea | uk
    pub cuisine: String,
    /// traditional | modern
    pub style: String,
    /// main | side | fast_food | snack | dessert | drink
    pub dish_type: String,
    pub meals: Vec<String>,
    /// pho | bun | mi | hu_tieu | banh_canh | lau | kho | khac
    pub soup_type: String,
    /// Giá tham khảo thấp nhất tại thị trường đang chọn (tiền tệ địa phương); `null` = gần như không bán ở thị trường này.
    pub price_min: Option<f64>,
    /// Giá tham khảo cao nhất tại thị trường đang chọn.
    pub price_max: Option<f64>,
    /// Mã tiền tệ của giá (VND, JPY, KRW, CNY, GBP).
    pub currency: Option<String>,
    /// Năng lượng ước tính cho một khẩu phần (kcal).
    pub calories_kcal: i32,
    pub description: String,
    pub image_url: String,
    pub featured: bool,
    /// Trọng số dùng cho weighted random.
    pub weight: f32,
    /// Điểm trung bình từ đánh giá của thành viên FoodRandom; `null` khi chưa có đánh giá.
    pub rating_avg: Option<f64>,
    pub rating_count: i32,
    pub favorite_count: i32,
}

/// Một dòng trong danh sách kết quả: món ăn + mức khớp bộ lọc + quán gần nhất (nếu có vị trí).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FoodListItem {
    #[serde(flatten)]
    pub food: FoodSummary,
    /// `exact`: thoả mọi tiêu chí lọc. `relaxed`: gợi ý mở rộng để trang đủ 12 món.
    pub match_level: String,
    /// Quán gần nhất phục vụ nền ẩm thực của món; `null` khi chưa có vị trí hoặc không tìm thấy.
    pub nearest_place: Option<Place>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LocationStatus {
    /// `disabled`: request không kèm vị trí. `ok`: đã gắn khoảng cách. `unavailable`: nhà cung cấp lỗi.
    pub status: String,
    #[serde(flatten)]
    pub source: Option<ProviderInfo>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FoodListResponse {
    pub items: Vec<FoodListItem>,
    pub page: usize,
    pub page_size: usize,
    /// Số món thoả mọi tiêu chí lọc.
    pub total: usize,
    pub total_pages: usize,
    /// Số gợi ý mở rộng được thêm vào trang này để đủ `page_size` món.
    pub relaxed_count: usize,
    pub sort: String,
    pub market: String,
    pub currency: String,
    pub lang: String,
    pub location: LocationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Ingredient {
    pub name: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Nutrition {
    pub calories_kcal: i32,
    pub protein_g: f32,
    pub carbs_g: f32,
    pub fat_g: f32,
    pub serving_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImageCredit {
    pub source_url: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
}

/// Chi tiết món ăn: mô tả, nguồn gốc, dinh dưỡng, nguyên liệu, công thức, món tương tự.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FoodDetail {
    #[serde(flatten)]
    pub food: FoodSummary,
    pub origin: String,
    pub cooking_style: String,
    pub prep_minutes: i32,
    pub cook_minutes: i32,
    pub servings: i32,
    pub nutrition: Nutrition,
    pub ingredients: Vec<Ingredient>,
    pub steps: Vec<String>,
    pub tags: Vec<String>,
    pub image_credit: ImageCredit,
    pub similar: Vec<FoodSummary>,
}

/// Dòng đầy đủ đọc từ bảng `foods`.
#[derive(Debug, Clone, FromRow)]
pub struct FoodRow {
    #[sqlx(flatten)]
    pub summary: FoodSummary,
    pub origin: String,
    pub cooking_style: String,
    pub prep_minutes: i32,
    pub cook_minutes: i32,
    pub servings: i32,
    pub serving_note: String,
    pub protein_g: f32,
    pub carbs_g: f32,
    pub fat_g: f32,
    pub ingredients: Value,
    pub steps: Value,
    pub tags: Vec<String>,
    pub image_source_url: Option<String>,
    pub image_author: Option<String>,
    pub image_license: Option<String>,
}

/// Bộ lọc món ăn đã chuẩn hoá. Dùng chung cho danh sách (GET) và random (POST).
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct FoodFilter {
    /// Thị trường (quốc gia): VN, JP, KR, CN, GB. Quyết định tiền tệ và mức ngân sách.
    #[serde(default)]
    pub market: String,
    /// Ngôn ngữ nội dung: vi, en, ja, ko, zh.
    #[serde(default)]
    pub lang: String,
    /// Bữa ăn: breakfast, lunch, dinner, snack. Chọn nhiều = khớp bất kỳ.
    #[serde(default)]
    pub meals: Vec<String>,
    /// Loại món: main, side, fast_food, snack, dessert, drink.
    pub dish_type: Option<String>,
    /// Loại ẩm thực: traditional, modern.
    pub style: Option<String>,
    /// Món nước: pho, bun, mi, hu_tieu, banh_canh, lau, kho.
    pub soup_type: Option<String>,
    /// Nền ẩm thực: vietnam, japan, china, korea, uk. Chọn nhiều = khớp bất kỳ.
    #[serde(default)]
    pub cuisines: Vec<String>,
    /// Mức ngân sách của thị trường: b1..b5 (xem `GET /meta/filters`). Chọn nhiều = khớp bất kỳ.
    #[serde(default)]
    pub budgets: Vec<String>,
    /// Khoảng giá tuỳ chỉnh theo tiền tệ của thị trường, dùng thay hoặc cùng `budgets`.
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    /// Từ khoá tìm theo tên món (không phân biệt dấu).
    pub q: Option<String>,
    /// Vị trí người dùng. Có `lat` thì phải có `lng`.
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    /// Bán kính tìm quán (km): 1, 3, 5, 10. Bỏ trống = không giới hạn.
    pub radius_km: Option<f64>,
}

fn check_enum(field: &str, value: &str, allowed: &[&str]) -> AppResult<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(AppError::validation(field, format!("Giá trị '{value}' không hợp lệ. Cho phép: {}", allowed.join(", "))))
    }
}

fn clean_list(field: &str, values: Vec<String>, allowed: &[&str]) -> AppResult<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for value in values {
        let value = value.trim().to_ascii_lowercase();
        if value.is_empty() || value == "all" {
            continue;
        }
        check_enum(field, &value, allowed)?;
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out.sort();
    Ok(out)
}

fn clean_option(field: &str, value: Option<String>, allowed: &[&str]) -> AppResult<Option<String>> {
    match value.map(|v| v.trim().to_ascii_lowercase()).filter(|v| !v.is_empty() && v != "all") {
        None => Ok(None),
        Some(v) => {
            check_enum(field, &v, allowed)?;
            Ok(Some(v))
        }
    }
}

impl FoodFilter {
    /// Kiểm tra và chuẩn hoá bộ lọc: bỏ giá trị "all", loại trùng, sắp xếp để khoá cache ổn định.
    pub fn normalize(self) -> AppResult<Self> {
        let filter = FoodFilter {
            market: i18n::parse_market(Some(&self.market))?.code.to_string(),
            lang: i18n::parse_locale(Some(&self.lang))?.to_string(),
            meals: clean_list("meals", self.meals, &MEALS)?,
            dish_type: clean_option("dish_type", self.dish_type, &DISH_TYPES)?,
            style: clean_option("style", self.style, &STYLES)?,
            soup_type: clean_option("soup_type", self.soup_type, &SOUP_TYPES)?,
            cuisines: clean_list("cuisines", self.cuisines, &CUISINES)?,
            budgets: clean_list("budgets", self.budgets, &BUDGET_CODES)?,
            price_min: self.price_min,
            price_max: self.price_max,
            q: self.q.map(|q| q.trim().to_string()).filter(|q| !q.is_empty()),
            lat: self.lat,
            lng: self.lng,
            radius_km: self.radius_km,
        };

        if let Some(q) = &filter.q {
            if q.chars().count() > 80 {
                return Err(AppError::validation("q", "Từ khoá tìm kiếm tối đa 80 ký tự"));
            }
        }
        if [filter.price_min, filter.price_max].into_iter().flatten().any(|v| !v.is_finite() || v < 0.0) {
            return Err(AppError::validation("price_min", "Khoảng giá không được âm"));
        }
        if let (Some(lo), Some(hi)) = (filter.price_min, filter.price_max) {
            if lo > hi {
                return Err(AppError::validation("price_min", "Giá thấp nhất phải nhỏ hơn hoặc bằng giá cao nhất"));
            }
        }
        match (filter.lat, filter.lng) {
            (Some(lat), Some(lng)) if valid_coords(lat, lng) => {}
            (None, None) => {}
            _ => return Err(AppError::validation("lat", "Vị trí không hợp lệ: cần cả lat và lng trong phạm vi toạ độ")),
        }
        if let Some(radius) = filter.radius_km {
            if !(radius.is_finite() && radius > 0.0 && radius <= 50.0) {
                return Err(AppError::validation("radius_km", "Bán kính phải lớn hơn 0 và không quá 50 km"));
            }
        }
        Ok(filter)
    }

    pub fn coords(&self) -> Option<(f64, f64)> {
        self.lat.zip(self.lng)
    }

    /// Phần bộ lọc quyết định tập món (không gồm vị trí) — dùng làm khoá cache.
    pub fn catalog_key(&self) -> String {
        format!(
            "mk={}|l={}|m={}|d={}|s={}|n={}|c={}|b={}|p={}-{}|q={}",
            self.market,
            self.lang,
            self.meals.join(","),
            self.dish_type.as_deref().unwrap_or(""),
            self.style.as_deref().unwrap_or(""),
            self.soup_type.as_deref().unwrap_or(""),
            self.cuisines.join(","),
            self.budgets.join(","),
            self.price_min.map(|v| v.to_string()).unwrap_or_default(),
            self.price_max.map(|v| v.to_string()).unwrap_or_default(),
            self.q.as_deref().map(crate::util::fold_vi).unwrap_or_default(),
        )
    }

    pub fn has_criteria(&self) -> bool {
        !self.meals.is_empty()
            || self.dish_type.is_some()
            || self.style.is_some()
            || self.soup_type.is_some()
            || !self.cuisines.is_empty()
            || !self.budgets.is_empty()
            || self.price_min.is_some()
            || self.price_max.is_some()
            || self.q.is_some()
    }
}

/// Tham số truy vấn của `GET /api/v1/foods`. Các danh sách viết cách nhau bởi dấu phẩy.
#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FoodListQuery {
    /// Thị trường: VN (mặc định), JP, KR, CN, GB.
    pub market: Option<String>,
    /// Ngôn ngữ: vi (mặc định), en, ja, ko, zh.
    pub lang: Option<String>,
    /// Bữa ăn, ví dụ `lunch,dinner`.
    pub meals: Option<String>,
    pub dish_type: Option<String>,
    pub style: Option<String>,
    pub soup_type: Option<String>,
    /// Nền ẩm thực, ví dụ `vietnam,japan`.
    pub cuisines: Option<String>,
    /// Mức ngân sách, ví dụ `b2,b3`.
    pub budgets: Option<String>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub q: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_km: Option<f64>,
    /// featured | nearest | price_asc | price_desc | rating | popular
    pub sort: Option<String>,
    /// Trang, bắt đầu từ 1.
    pub page: Option<usize>,
    /// Số món mỗi trang (mặc định 12, tối đa 48).
    pub page_size: Option<usize>,
}

fn split_csv(value: Option<String>) -> Vec<String> {
    value.map(|v| v.split(',').map(str::to_string).collect()).unwrap_or_default()
}

impl FoodListQuery {
    pub fn into_parts(self) -> AppResult<(FoodFilter, Paging)> {
        let paging = Paging::new(self.sort, self.page, self.page_size)?;
        let filter = FoodFilter {
            market: self.market.unwrap_or_default(),
            lang: self.lang.unwrap_or_default(),
            meals: split_csv(self.meals),
            dish_type: self.dish_type,
            style: self.style,
            soup_type: self.soup_type,
            cuisines: split_csv(self.cuisines),
            budgets: split_csv(self.budgets),
            price_min: self.price_min,
            price_max: self.price_max,
            q: self.q,
            lat: self.lat,
            lng: self.lng,
            radius_km: self.radius_km,
        }
        .normalize()?;
        Ok((filter, paging))
    }
}

#[derive(Debug, Clone)]
pub struct Paging {
    /// `None` = để service chọn mặc định (gần nhất nếu có vị trí, nổi bật nếu không).
    pub sort: Option<String>,
    pub page: usize,
    pub page_size: usize,
}

impl Paging {
    pub fn new(sort: Option<String>, page: Option<usize>, page_size: Option<usize>) -> AppResult<Self> {
        let sort = clean_option("sort", sort, &SORTS)?;
        let page = page.unwrap_or(1);
        if page == 0 || page > 10_000 {
            return Err(AppError::validation("page", "Số trang phải từ 1 trở lên"));
        }
        let page_size = page_size.unwrap_or(PAGE_SIZE);
        if page_size == 0 || page_size > MAX_PAGE_SIZE {
            return Err(AppError::validation("page_size", format!("Số món mỗi trang phải từ 1 đến {MAX_PAGE_SIZE}")));
        }
        Ok(Self { sort, page, page_size })
    }
}

/// Yêu cầu chọn ngẫu nhiên một món. Việc chọn diễn ra hoàn toàn ở backend.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RandomRequest {
    /// Bộ lọc hiện tại của người dùng (giống tham số của danh sách).
    #[serde(default)]
    pub filter: FoodFilter,
    /// Cách sắp xếp đang dùng ở danh sách; cần để xác định đúng 12 món của trang.
    pub sort: Option<String>,
    /// Trang đang xem. Random chọn trong các món của trang này.
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    /// `page` (mặc định): chọn trong các món của trang đang xem. `all`: chọn trong mọi món khớp bộ lọc.
    pub scope: Option<String>,
    /// Slug các món vừa được chọn, để tránh lặp lại.
    #[serde(default)]
    #[validate(length(max = 50, message = "Tối đa 50 món loại trừ"))]
    pub exclude: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RandomResponse {
    pub food: FoodListItem,
    /// Số món trong tập mà kết quả được chọn ra.
    pub pool_size: usize,
    /// `page` hoặc `all`.
    pub scope: String,
    pub page: usize,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RatingRequest {
    #[validate(range(min = 1, max = 5, message = "Điểm đánh giá từ 1 đến 5"))]
    pub score: i16,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RatingResponse {
    pub rating_avg: Option<f64>,
    pub rating_count: i32,
    /// Điểm của người dùng hiện tại; `null` nếu chưa đánh giá hoặc chưa đăng nhập.
    pub mine: Option<i16>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OptionCount {
    pub code: String,
    pub label: String,
    pub count: i64,
}

/// Danh mục bộ lọc kèm số món của từng nền ẩm thực và cấu hình các thị trường.
#[derive(Debug, Serialize, ToSchema)]
pub struct FilterMeta {
    pub total_foods: i64,
    pub cuisines: Vec<OptionCount>,
    pub page_size: usize,
    pub markets: Vec<i18n::Market>,
    pub locales: Vec<&'static str>,
}

/// Tham số ngôn ngữ và thị trường dùng chung cho các API trả món ăn.
#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LocaleQuery {
    /// Ngôn ngữ: vi (mặc định), en, ja, ko, zh.
    pub lang: Option<String>,
    /// Thị trường: VN (mặc định), JP, KR, CN, GB.
    pub market: Option<String>,
}

impl LocaleQuery {
    pub fn parse(&self) -> AppResult<(&'static str, &'static i18n::Market)> {
        Ok((i18n::parse_locale(self.lang.as_deref())?, i18n::parse_market(self.market.as_deref())?))
    }
}
