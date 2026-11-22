use sqlx::{PgPool, Postgres, QueryBuilder};

use super::dto::{FoodFilter, FoodRow, FoodSummary};
use crate::{i18n, util::fold_vi};

/// Cột của bản tóm tắt món. Bảng `foods` là `f`, giá theo thị trường là `p`, bản dịch là `t`
/// (bản dịch thiếu thì rơi về tiếng Việt gốc).
const SUMMARY_COLUMNS: &str =
    "f.id, f.slug, COALESCE(t.name, f.name) AS name, f.original_name, f.cuisine, f.style, \
     f.dish_type, f.meals, f.soup_type, p.price_min, p.price_max, p.currency, f.calories_kcal, \
     COALESCE(t.description, f.description) AS description, f.image_url, f.featured, f.weight, \
     CASE WHEN f.rating_count > 0 THEN f.rating_sum::FLOAT8 / f.rating_count END AS rating_avg, \
     f.rating_count, f.favorite_count";

const DETAIL_COLUMNS: &str = "COALESCE(t.origin, f.origin) AS origin, COALESCE(t.cooking_style, f.cooking_style) AS cooking_style, \
     f.prep_minutes, f.cook_minutes, f.servings, COALESCE(t.serving_note, f.serving_note) AS serving_note, \
     f.protein_g, f.carbs_g, f.fat_g, COALESCE(t.ingredients, f.ingredients) AS ingredients, \
     COALESCE(t.steps, f.steps) AS steps, COALESCE(t.tags, f.tags) AS tags, \
     f.image_source_url, f.image_author, f.image_license";

/// `FROM foods f LEFT JOIN food_prices p … LEFT JOIN food_translations t …` với tham số được bind an toàn.
pub fn push_from(qb: &mut QueryBuilder<'_, Postgres>, market: &str, lang: &str) {
    qb.push(" FROM foods f LEFT JOIN food_prices p ON p.food_id = f.id AND p.market = ")
        .push_bind(market.to_string())
        .push(" LEFT JOIN food_translations t ON t.food_id = f.id AND t.locale = ")
        .push_bind(lang.to_string());
}

/// Truy cập dữ liệu bảng `foods`, `food_prices` và `food_translations`.
#[derive(Clone)]
pub struct FoodRepository {
    pool: PgPool,
}

impl FoodRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Mọi món thoả bộ lọc (không phân trang). Việc sắp xếp theo khoảng cách và cắt trang do service làm,
    /// vì khoảng cách phụ thuộc vị trí từng người dùng.
    pub async fn search(&self, filter: &FoodFilter) -> sqlx::Result<Vec<FoodSummary>> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(format!("SELECT {SUMMARY_COLUMNS}"));
        push_from(&mut qb, &filter.market, &filter.lang);
        qb.push(" WHERE TRUE");

        if !filter.meals.is_empty() {
            qb.push(" AND f.meals && ").push_bind(filter.meals.clone());
        }
        if let Some(dish_type) = &filter.dish_type {
            qb.push(" AND f.dish_type = ").push_bind(dish_type.clone());
        }
        if let Some(style) = &filter.style {
            qb.push(" AND f.style = ").push_bind(style.clone());
        }
        if let Some(soup_type) = &filter.soup_type {
            qb.push(" AND f.soup_type = ").push_bind(soup_type.clone());
        }
        if !filter.cuisines.is_empty() {
            qb.push(" AND f.cuisine = ANY(")
                .push_bind(filter.cuisines.clone())
                .push(")");
        }

        let market = i18n::market(&filter.market);
        let bands: Vec<i18n::BudgetBand> = filter
            .budgets
            .iter()
            .filter_map(|code| market.and_then(|m| m.band(code)))
            .collect();
        if !bands.is_empty() {
            // Món thuộc một mức ngân sách khi khoảng giá của món giao với mức đó (không tính chạm biên).
            // Món không có giá ở thị trường này (p.* là NULL) tự bị loại.
            qb.push(" AND (");
            for (i, band) in bands.iter().enumerate() {
                if i > 0 {
                    qb.push(" OR ");
                }
                qb.push("(GREATEST(p.price_max, p.price_min + 0.01) > ")
                    .push_bind(band.min);
                if let Some(max) = band.max {
                    qb.push(" AND p.price_min < ").push_bind(max);
                }
                qb.push(")");
            }
            qb.push(")");
        }
        if let Some(price_min) = filter.price_min {
            qb.push(" AND p.price_max >= ").push_bind(price_min);
        }
        if let Some(price_max) = filter.price_max {
            qb.push(" AND p.price_min <= ").push_bind(price_max);
        }
        if let Some(q) = &filter.q {
            let pattern = format!(
                "%{}%",
                fold_vi(q)
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            );
            qb.push(" AND f.search_text LIKE ").push_bind(pattern);
        }

        qb.push(" ORDER BY f.featured DESC, (p.price_min IS NULL), name ASC");
        qb.build_query_as::<FoodSummary>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_by_slug(
        &self,
        slug: &str,
        market: &str,
        lang: &str,
    ) -> sqlx::Result<Option<FoodRow>> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("SELECT {SUMMARY_COLUMNS}, {DETAIL_COLUMNS}"));
        push_from(&mut qb, market, lang);
        qb.push(" WHERE f.slug = ").push_bind(slug.to_string());
        qb.build_query_as::<FoodRow>()
            .fetch_optional(&self.pool)
            .await
    }

    /// Món tương tự: cùng nền ẩm thực, ưu tiên cùng loại món và cùng nhóm món nước.
    pub async fn similar(
        &self,
        food: &FoodSummary,
        market: &str,
        lang: &str,
        limit: i64,
    ) -> sqlx::Result<Vec<FoodSummary>> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(format!("SELECT {SUMMARY_COLUMNS}"));
        push_from(&mut qb, market, lang);
        qb.push(" WHERE f.id <> ")
            .push_bind(food.id)
            .push(" AND (f.cuisine = ")
            .push_bind(food.cuisine.clone())
            .push(" OR f.dish_type = ")
            .push_bind(food.dish_type.clone())
            .push(") ORDER BY (f.cuisine = ")
            .push_bind(food.cuisine.clone())
            .push(")::INT + (f.dish_type = ")
            .push_bind(food.dish_type.clone())
            .push(")::INT + (f.soup_type = ")
            .push_bind(food.soup_type.clone())
            .push(")::INT DESC, f.featured DESC, name ASC LIMIT ")
            .push_bind(limit);
        qb.build_query_as::<FoodSummary>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count_by_cuisine(&self) -> sqlx::Result<Vec<(String, i64)>> {
        sqlx::query_as::<_, (String, i64)>("SELECT cuisine, COUNT(*) FROM foods GROUP BY cuisine")
            .fetch_all(&self.pool)
            .await
    }
}
