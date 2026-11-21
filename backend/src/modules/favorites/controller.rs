use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, put},
    Json, Router,
};
use uuid::Uuid;

use super::dto::{
    FavoriteFoodSlugs, FavoriteFoodsResponse, FavoritePlace, FavoritePlacesResponse, FavoriteToggleResponse, SavePlaceRequest,
};
use crate::{
    error::{AppResult, ErrorBody},
    extract::{AuthUser, ValidatedJson},
    modules::foods::dto::LocaleQuery,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/favorites/foods", get(list_foods))
        .route("/favorites/foods/slugs", get(list_food_slugs))
        .route("/favorites/foods/{slug}", put(add_food).delete(remove_food))
        .route("/favorites/places", get(list_places).post(save_place))
        .route("/favorites/places/{id}", delete(remove_place))
}

/// Danh sách món yêu thích
#[utoipa::path(
    get, path = "/api/v1/favorites/foods", tag = "Favorites",
    params(LocaleQuery),
    responses((status = 200, body = FavoriteFoodsResponse), (status = 401, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn list_foods(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(locale): Query<LocaleQuery>,
) -> AppResult<Json<FavoriteFoodsResponse>> {
    let (lang, market) = locale.parse()?;
    Ok(Json(FavoriteFoodsResponse { items: state.favorites.foods(user_id, market.code, lang).await? }))
}

/// Slug các món yêu thích (để tô nút trái tim trên thẻ món)
#[utoipa::path(
    get, path = "/api/v1/favorites/foods/slugs", tag = "Favorites",
    responses((status = 200, body = FavoriteFoodSlugs), (status = 401, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn list_food_slugs(State(state): State<AppState>, AuthUser(user_id): AuthUser) -> AppResult<Json<FavoriteFoodSlugs>> {
    Ok(Json(FavoriteFoodSlugs { slugs: state.favorites.food_slugs(user_id).await? }))
}

/// Lưu món yêu thích
#[utoipa::path(
    put, path = "/api/v1/favorites/foods/{slug}", tag = "Favorites",
    params(("slug" = String, Path, description = "Slug của món ăn")),
    responses((status = 200, body = FavoriteToggleResponse), (status = 401, body = ErrorBody), (status = 404, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn add_food(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(slug): Path<String>,
) -> AppResult<Json<FavoriteToggleResponse>> {
    state.favorites.add_food(user_id, &slug).await?;
    Ok(Json(FavoriteToggleResponse { slug, favorited: true }))
}

/// Bỏ món yêu thích
#[utoipa::path(
    delete, path = "/api/v1/favorites/foods/{slug}", tag = "Favorites",
    params(("slug" = String, Path, description = "Slug của món ăn")),
    responses((status = 200, body = FavoriteToggleResponse), (status = 401, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn remove_food(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(slug): Path<String>,
) -> AppResult<Json<FavoriteToggleResponse>> {
    state.favorites.remove_food(user_id, &slug).await?;
    Ok(Json(FavoriteToggleResponse { slug, favorited: false }))
}

/// Danh sách quán ăn yêu thích
#[utoipa::path(
    get, path = "/api/v1/favorites/places", tag = "Favorites",
    responses((status = 200, body = FavoritePlacesResponse), (status = 401, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn list_places(State(state): State<AppState>, AuthUser(user_id): AuthUser) -> AppResult<Json<FavoritePlacesResponse>> {
    Ok(Json(FavoritePlacesResponse { items: state.favorites.places(user_id).await? }))
}

/// Lưu quán ăn yêu thích
///
/// Thông tin quán lấy từ kết quả của `GET /api/v1/location/nearby`.
#[utoipa::path(
    post, path = "/api/v1/favorites/places", tag = "Favorites",
    request_body = SavePlaceRequest,
    responses((status = 201, body = FavoritePlace), (status = 401, body = ErrorBody), (status = 422, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn save_place(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(body): ValidatedJson<SavePlaceRequest>,
) -> AppResult<(StatusCode, Json<FavoritePlace>)> {
    Ok((StatusCode::CREATED, Json(state.favorites.save_place(user_id, body).await?)))
}

/// Xoá quán ăn yêu thích
#[utoipa::path(
    delete, path = "/api/v1/favorites/places/{id}", tag = "Favorites",
    params(("id" = Uuid, Path, description = "Mã bản ghi quán đã lưu")),
    responses((status = 204), (status = 401, body = ErrorBody), (status = 404, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn remove_place(State(state): State<AppState>, AuthUser(user_id): AuthUser, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    state.favorites.remove_place(user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
