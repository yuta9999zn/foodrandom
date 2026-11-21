use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use super::dto::{FavoritePlace, SavePlaceRequest};
use crate::modules::foods::{dto::FoodSummary, repository as food_repo};

const PLACE_COLUMNS: &str = "p.id, p.provider, p.place_ref, p.name, p.address, p.lat, p.lng, p.cuisine, \
     f.slug AS food_slug, f.name AS food_name, p.maps_url, p.note, p.created_at";

#[derive(Clone)]
pub struct FavoriteRepository {
    pool: PgPool,
}

impl FavoriteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn foods(&self, user_id: Uuid, market: &str, lang: &str) -> sqlx::Result<Vec<FoodSummary>> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(format!("SELECT {}", food_repo::summary_columns()));
        food_repo::push_from(&mut qb, market, lang);
        qb.push(" JOIN favorite_foods ff ON ff.food_id = f.id WHERE ff.user_id = ")
            .push_bind(user_id)
            .push(" ORDER BY ff.created_at DESC");
        qb.build_query_as::<FoodSummary>().fetch_all(&self.pool).await
    }

    pub async fn food_slugs(&self, user_id: Uuid) -> sqlx::Result<Vec<String>> {
        sqlx::query_scalar(
            "SELECT f.slug FROM favorite_foods ff JOIN foods f ON f.id = ff.food_id WHERE ff.user_id = $1 ORDER BY ff.created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Thêm món yêu thích. Trả `None` nếu slug không tồn tại. Bộ đếm trên món chỉ tăng khi thực sự thêm mới.
    pub async fn add_food(&self, user_id: Uuid, slug: &str) -> sqlx::Result<Option<()>> {
        let mut tx = self.pool.begin().await?;
        let Some(food_id) = sqlx::query_scalar::<_, Uuid>("SELECT id FROM foods WHERE slug = $1").bind(slug).fetch_optional(&mut *tx).await?
        else {
            return Ok(None);
        };
        let inserted = sqlx::query("INSERT INTO favorite_foods (user_id, food_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(user_id)
            .bind(food_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        if inserted > 0 {
            sqlx::query("UPDATE foods SET favorite_count = favorite_count + 1 WHERE id = $1").bind(food_id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(Some(()))
    }

    pub async fn remove_food(&self, user_id: Uuid, slug: &str) -> sqlx::Result<()> {
        let mut tx = self.pool.begin().await?;
        let removed: Option<Uuid> = sqlx::query_scalar(
            "DELETE FROM favorite_foods ff USING foods f \
             WHERE ff.food_id = f.id AND ff.user_id = $1 AND f.slug = $2 RETURNING f.id",
        )
        .bind(user_id)
        .bind(slug)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(food_id) = removed {
            sqlx::query("UPDATE foods SET favorite_count = GREATEST(favorite_count - 1, 0) WHERE id = $1")
                .bind(food_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await
    }

    pub async fn places(&self, user_id: Uuid) -> sqlx::Result<Vec<FavoritePlace>> {
        sqlx::query_as::<_, FavoritePlace>(&format!(
            "SELECT {PLACE_COLUMNS} FROM favorite_places p LEFT JOIN foods f ON f.id = p.food_id \
             WHERE p.user_id = $1 ORDER BY p.created_at DESC"
        ))
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn count_places(&self, user_id: Uuid) -> sqlx::Result<i64> {
        sqlx::query_scalar("SELECT COUNT(*) FROM favorite_places WHERE user_id = $1").bind(user_id).fetch_one(&self.pool).await
    }

    /// Lưu quán yêu thích; lưu lại cùng một quán thì cập nhật thông tin thay vì tạo bản ghi trùng.
    pub async fn save_place(&self, user_id: Uuid, req: &SavePlaceRequest) -> sqlx::Result<FavoritePlace> {
        sqlx::query_as::<_, FavoritePlace>(&format!(
            "WITH saved AS ( \
                 INSERT INTO favorite_places (user_id, provider, place_ref, name, address, lat, lng, cuisine, food_id, maps_url, note) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, (SELECT id FROM foods WHERE slug = $9), $10, $11) \
                 ON CONFLICT (user_id, provider, place_ref) DO UPDATE SET \
                     name = EXCLUDED.name, address = EXCLUDED.address, lat = EXCLUDED.lat, lng = EXCLUDED.lng, \
                     cuisine = COALESCE(EXCLUDED.cuisine, favorite_places.cuisine), \
                     food_id = COALESCE(EXCLUDED.food_id, favorite_places.food_id), \
                     maps_url = EXCLUDED.maps_url, note = COALESCE(EXCLUDED.note, favorite_places.note) \
                 RETURNING * \
             ) SELECT {PLACE_COLUMNS} FROM saved p LEFT JOIN foods f ON f.id = p.food_id"
        ))
        .bind(user_id)
        .bind(&req.provider)
        .bind(&req.place_ref)
        .bind(req.name.trim())
        .bind(req.address.as_deref().map(str::trim).filter(|a| !a.is_empty()))
        .bind(req.lat)
        .bind(req.lng)
        .bind(&req.cuisine)
        .bind(&req.food_slug)
        .bind(&req.maps_url)
        .bind(req.note.as_deref().map(str::trim).filter(|n| !n.is_empty()))
        .fetch_one(&self.pool)
        .await
    }

    pub async fn remove_place(&self, user_id: Uuid, id: Uuid) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM favorite_places WHERE id = $1 AND user_id = $2").bind(id).bind(user_id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }
}
