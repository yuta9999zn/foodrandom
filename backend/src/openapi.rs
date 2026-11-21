use axum::{response::Html, Json};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::modules::{auth, favorites, foods, location};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FoodRandom API",
        version = "0.1.0",
        description = "API của FoodRandom — khám phá và chọn ngẫu nhiên món ăn theo bữa, loại món, phong cách, ngân sách và vị trí.\n\nPhiên đăng nhập là cookie httpOnly `fr_session` (JWT). Lỗi luôn có dạng `{ \"error\": { \"code\", \"message\", \"details\" } }`."
    ),
    paths(
        foods::controller::list_foods,
        foods::controller::get_food,
        foods::controller::get_rating,
        foods::controller::put_rating,
        foods::controller::random_food,
        foods::controller::filter_meta,
        location::controller::provider_info,
        location::controller::nearby,
        location::controller::geocode,
        location::controller::reverse,
        auth::controller::request_register_otp,
        auth::controller::verify_register,
        auth::controller::login,
        auth::controller::logout,
        auth::controller::me,
        auth::controller::update_me,
        auth::controller::forgot_password,
        auth::controller::reset_password,
        auth::controller::change_password,
        favorites::controller::list_foods,
        favorites::controller::list_food_slugs,
        favorites::controller::add_food,
        favorites::controller::remove_food,
        favorites::controller::list_places,
        favorites::controller::save_place,
        favorites::controller::remove_place,
    ),
    modifiers(&SessionCookie),
    tags(
        (name = "Foods", description = "Danh sách, chi tiết và đánh giá món ăn"),
        (name = "Random", description = "Random Engine — chọn ngẫu nhiên ở backend"),
        (name = "Location", description = "Tìm quán quanh người dùng, địa chỉ ⇄ toạ độ"),
        (name = "Auth", description = "Đăng ký bằng OTP email, đăng nhập, mật khẩu"),
        (name = "Favorites", description = "Món ăn và quán ăn yêu thích"),
    )
)]
pub struct ApiDoc;

struct SessionCookie;

impl Modify for SessionCookie {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "session_cookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(auth::session::COOKIE_NAME))),
        );
    }
}

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Trang Swagger UI. Tài nguyên giao diện tải từ CDN; đặc tả lấy từ `/api-docs/openapi.json`.
pub async fn swagger_ui() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="vi">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>FoodRandom API — Swagger UI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.17.14/swagger-ui.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5.17.14/swagger-ui-bundle.js"></script>
  <script>
    window.ui = SwaggerUIBundle({ url: '/api-docs/openapi.json', dom_id: '#swagger-ui', deepLinking: true, withCredentials: true });
  </script>
</body>
</html>"#,
    )
}
