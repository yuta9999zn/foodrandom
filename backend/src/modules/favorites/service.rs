use uuid::Uuid;

use super::{
    dto::{FavoritePlace, SavePlaceRequest},
    repository::FavoriteRepository,
};
use crate::{
    error::{AppError, AppResult},
    modules::foods::dto::{FoodSummary, CUISINES},
};

const MAX_PLACES_PER_USER: i64 = 200;
const PROVIDERS: [&str; 3] = ["google", "osm", "mock"];

/// Món ăn yêu thích và quán ăn yêu thích của thành viên.
#[derive(Clone)]
pub struct FavoriteService {
    repo: FavoriteRepository,
}

impl FavoriteService {
    pub fn new(repo: FavoriteRepository) -> Self {
        Self { repo }
    }

    pub async fn foods(&self, user_id: Uuid, market: &str, lang: &str) -> AppResult<Vec<FoodSummary>> {
        Ok(self.repo.foods(user_id, market, lang).await?)
    }

    pub async fn food_slugs(&self, user_id: Uuid) -> AppResult<Vec<String>> {
        Ok(self.repo.food_slugs(user_id).await?)
    }

    pub async fn add_food(&self, user_id: Uuid, slug: &str) -> AppResult<()> {
        self.repo.add_food(user_id, slug).await?.ok_or_else(|| AppError::NotFound("Không tìm thấy món ăn".into()))
    }

    pub async fn remove_food(&self, user_id: Uuid, slug: &str) -> AppResult<()> {
        Ok(self.repo.remove_food(user_id, slug).await?)
    }

    pub async fn places(&self, user_id: Uuid) -> AppResult<Vec<FavoritePlace>> {
        Ok(self.repo.places(user_id).await?)
    }

    pub async fn save_place(&self, user_id: Uuid, request: SavePlaceRequest) -> AppResult<FavoritePlace> {
        if !PROVIDERS.contains(&request.provider.as_str()) {
            return Err(AppError::validation("provider", "Nhà cung cấp địa điểm không hợp lệ"));
        }
        if let Some(cuisine) = &request.cuisine {
            if !CUISINES.contains(&cuisine.as_str()) {
                return Err(AppError::validation("cuisine", "Nền ẩm thực không hợp lệ"));
            }
        }
        if self.repo.count_places(user_id).await? >= MAX_PLACES_PER_USER {
            return Err(AppError::BadRequest(format!("Bạn chỉ lưu được tối đa {MAX_PLACES_PER_USER} quán yêu thích")));
        }
        Ok(self.repo.save_place(user_id, &request).await?)
    }

    pub async fn remove_place(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        if self.repo.remove_place(user_id, id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound("Không tìm thấy quán đã lưu".into()))
        }
    }
}
