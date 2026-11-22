use axum::{response::Html, Json};
use utoipa::OpenApi;

use crate::modules::{foods, location};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FoodRandom API",
        version = "0.2.0",
        description = "API của FoodRandom — khám phá và chọn ngẫu nhiên món ăn theo bữa, loại món, phong cách, ngân sách và vị trí.\n\nAPI công khai, không cần đăng nhập. Lỗi luôn có dạng `{ \"error\": { \"code\", \"message\", \"details\" } }`."
    ),
    paths(
        foods::controller::list_foods,
        foods::controller::get_food,
        foods::controller::random_food,
        foods::controller::filter_meta,
        location::controller::provider_info,
        location::controller::nearby,
        location::controller::geocode,
        location::controller::reverse,
    ),
    tags(
        (name = "Foods", description = "Danh sách và chi tiết món ăn"),
        (name = "Random", description = "Random Engine — chọn ngẫu nhiên ở backend"),
        (name = "Location", description = "Tìm quán quanh người dùng, địa chỉ ⇄ toạ độ"),
    )
)]
pub struct ApiDoc;

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
