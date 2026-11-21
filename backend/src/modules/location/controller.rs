use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};

use super::dto::{GeocodeQuery, GeocodeResponse, NearbyQuery, NearbyResponse, ProviderInfo, ReverseQuery, ReverseResponse};
use crate::{
    error::{AppError, AppResult, ErrorBody},
    extract::ClientIp,
    i18n,
    modules::{foods::dto::CUISINES, location::service::NearbyParams},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/location/provider", get(provider_info))
        .route("/location/nearby", get(nearby))
        .route("/location/geocode", get(geocode))
        .route("/location/reverse", get(reverse))
}

/// Các lời gọi địa điểm đi ra dịch vụ ngoài nên giới hạn theo IP.
async fn limit(state: &AppState, ip: &str) -> AppResult<()> {
    state
        .cache
        .rate_limit(&format!("rl:location:{ip}"), 40, 60, "Bạn tìm địa điểm quá nhanh, vui lòng thử lại sau ít phút")
        .await
}

/// Từ khoá tên quán suy ra từ tên món Việt: "Phở bò Nam Định" → "Phở", "Bún bò Huế" → "Bún bò".
fn keyword_from_food_name(name: &str) -> Option<String> {
    let base = name.split('(').next().unwrap_or(name);
    let words: Vec<&str> = base.split_whitespace().collect();
    let first = words.first()?.to_lowercase();
    let take = if matches!(first.as_str(), "phở" | "lẩu" | "xôi" | "chè" | "cháo") { 1 } else { 2 };
    Some(words.iter().take(take).copied().collect::<Vec<_>>().join(" "))
}

/// Nhà cung cấp địa điểm đang dùng
#[utoipa::path(get, path = "/api/v1/location/provider", tag = "Location", responses((status = 200, body = ProviderInfo)))]
pub async fn provider_info(State(state): State<AppState>) -> Json<ProviderInfo> {
    Json(state.location.info())
}

/// Tìm quán ăn quanh một vị trí
///
/// Truyền `food` (slug món) để tìm quán phục vụ món đó, hoặc `cuisine` để lọc theo nền ẩm thực.
/// Kết quả sắp theo khoảng cách tăng dần, tính từ toạ độ gửi lên.
#[utoipa::path(
    get, path = "/api/v1/location/nearby", tag = "Location",
    params(NearbyQuery),
    responses(
        (status = 200, body = NearbyResponse),
        (status = 422, body = ErrorBody),
        (status = 429, body = ErrorBody),
        (status = 503, body = ErrorBody, description = "Nhà cung cấp địa điểm không phản hồi"),
    )
)]
pub async fn nearby(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Query(query): Query<NearbyQuery>,
) -> AppResult<Json<NearbyResponse>> {
    limit(&state, &ip).await?;

    let mut cuisines: Vec<String> = Vec::new();
    for code in query.cuisine.as_deref().unwrap_or("").split(',').map(str::trim).filter(|c| !c.is_empty()) {
        if !CUISINES.contains(&code) {
            return Err(AppError::validation("cuisine", format!("Nền ẩm thực '{code}' không hợp lệ")));
        }
        cuisines.push(code.to_string());
    }
    if let Some(radius) = query.radius_km {
        if !(radius.is_finite() && radius > 0.0 && radius <= 50.0) {
            return Err(AppError::validation("radius_km", "Bán kính phải lớn hơn 0 và không quá 50 km"));
        }
    }

    let market = i18n::parse_market(query.market.as_deref())?;
    let lang = i18n::parse_locale(query.lang.as_deref())?;

    let mut keyword = None;
    if let Some(slug) = query.food.as_deref().filter(|s| !s.is_empty()) {
        // Tên món bằng ngôn ngữ của thị trường: người ở Nhật tìm "Phở" sẽ tra "フォー" quanh vị trí ở Nhật.
        let food = state.foods.detail(slug, market.language, market).await?.food;
        keyword = match (state.location.provider_name(), food.cuisine.as_str(), market.code) {
            ("google", _, _) => Some(food.name.clone()),
            // OpenStreetMap: ở Việt Nam tên quán thường chứa tên món ("Phở Hoà"); nơi khác lọc theo thẻ ẩm thực.
            (_, "vietnam", "VN") => keyword_from_food_name(&food.name),
            _ => None,
        };
        cuisines = vec![food.cuisine];
    }

    let limit = query.limit.unwrap_or(12).clamp(1, 30);
    let by_rating = match query.sort.as_deref().unwrap_or("distance") {
        "distance" => false,
        "rating" => true,
        other => return Err(AppError::validation("sort", format!("Giá trị '{other}' không hợp lệ. Cho phép: distance, rating"))),
    };
    let places = state
        .location
        .nearby(NearbyParams {
            lat: query.lat,
            lng: query.lng,
            radius_km: query.radius_km,
            cuisines: &cuisines,
            keyword: keyword.as_deref(),
            limit,
            market,
            language: lang,
            by_rating,
        })
        .await?;
    let has_ratings = state.location.has_ratings();
    let sort = if by_rating && has_ratings { "rating" } else { "distance" };
    Ok(Json(NearbyResponse { source: state.location.info(), sort: sort.into(), has_ratings, places }))
}

/// Đổi địa chỉ nhập tay thành toạ độ
#[utoipa::path(
    get, path = "/api/v1/location/geocode", tag = "Location",
    params(GeocodeQuery),
    responses((status = 200, body = GeocodeResponse), (status = 422, body = ErrorBody), (status = 503, body = ErrorBody))
)]
pub async fn geocode(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Query(query): Query<GeocodeQuery>,
) -> AppResult<Json<GeocodeResponse>> {
    limit(&state, &ip).await?;
    let market = i18n::parse_market(query.market.as_deref())?;
    let lang = i18n::parse_locale(query.lang.as_deref())?;
    let results = state.location.geocode(&query.q, market, lang).await?;
    Ok(Json(GeocodeResponse { source: state.location.info(), results }))
}

/// Đổi toạ độ thành tên khu vực
#[utoipa::path(
    get, path = "/api/v1/location/reverse", tag = "Location",
    params(ReverseQuery),
    responses((status = 200, body = ReverseResponse), (status = 422, body = ErrorBody), (status = 503, body = ErrorBody))
)]
pub async fn reverse(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Query(query): Query<ReverseQuery>,
) -> AppResult<Json<ReverseResponse>> {
    limit(&state, &ip).await?;
    let lang = i18n::parse_locale(query.lang.as_deref())?;
    let label = state.location.reverse(query.lat, query.lng, lang).await?;
    Ok(Json(ReverseResponse { source: state.location.info(), label }))
}

#[cfg(test)]
mod tests {
    use super::keyword_from_food_name;

    #[test]
    fn derives_place_keyword_from_dish_name() {
        assert_eq!(keyword_from_food_name("Phở bò Nam Định").as_deref(), Some("Phở"));
        assert_eq!(keyword_from_food_name("Bún bò Huế").as_deref(), Some("Bún bò"));
        assert_eq!(keyword_from_food_name("Cơm tấm sườn bì chả").as_deref(), Some("Cơm tấm"));
    }
}
