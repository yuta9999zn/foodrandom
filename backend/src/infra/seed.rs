use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    modules::{
        foods::dto::{CUISINES, DISH_TYPES, MEALS, SOUP_TYPES, STYLES},
        location::provider::cuisine_label_vi,
    },
    util::fold_vi,
};

#[derive(Debug, Deserialize)]
struct SeedIngredient {
    name: String,
    amount: String,
}

#[derive(Debug, Deserialize)]
struct SeedNutrition {
    protein_g: f32,
    carbs_g: f32,
    fat_g: f32,
}

#[derive(Debug, Deserialize)]
struct SeedImage {
    file: String,
    source_url: Option<String>,
    author: Option<String>,
    license: Option<String>,
}

/// Một món trong tệp seed `seed/foods/<cuisine>.json`.
#[derive(Debug, Deserialize)]
struct SeedFood {
    slug: String,
    name: String,
    original_name: Option<String>,
    cuisine: String,
    style: String,
    dish_type: String,
    meals: Vec<String>,
    soup_type: String,
    price_min: i32,
    price_max: i32,
    calories_kcal: i32,
    #[serde(default)]
    serving_note: String,
    nutrition: SeedNutrition,
    description: String,
    #[serde(default)]
    origin: String,
    #[serde(default)]
    cooking_style: String,
    #[serde(default)]
    prep_minutes: i32,
    #[serde(default)]
    cook_minutes: i32,
    #[serde(default = "one")]
    servings: i32,
    ingredients: Vec<SeedIngredient>,
    steps: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    image: SeedImage,
    #[serde(default)]
    featured: bool,
    weight: Option<f32>,
}

fn one() -> i32 {
    1
}

impl SeedFood {
    fn validate(&self) -> Result<()> {
        let check = |field: &str, value: &str, allowed: &[&str]| -> Result<()> {
            if allowed.contains(&value) {
                Ok(())
            } else {
                bail!("món '{}': {field}='{value}' không hợp lệ", self.slug)
            }
        };
        check("cuisine", &self.cuisine, &CUISINES)?;
        check("style", &self.style, &STYLES)?;
        check("dish_type", &self.dish_type, &DISH_TYPES)?;
        check("soup_type", &self.soup_type, &SOUP_TYPES)?;
        if self.meals.is_empty() {
            bail!("món '{}': thiếu meals", self.slug);
        }
        for meal in &self.meals {
            check("meals", meal, &MEALS)?;
        }
        if self.price_min < 0 || self.price_max < self.price_min {
            bail!("món '{}': khoảng giá không hợp lệ", self.slug);
        }
        if self.slug.is_empty() || !self.slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') {
            bail!("slug '{}' không hợp lệ", self.slug);
        }
        Ok(())
    }
}

/// Nạp dữ liệu món ăn từ thư mục seed. Upsert theo slug nên chạy lại nhiều lần (hoặc trên nhiều bản sao
/// backend cùng lúc) đều an toàn; điểm đánh giá và lượt yêu thích hiện có được giữ nguyên.
pub async fn seed_foods(pool: &PgPool, dir: &str) -> Result<usize> {
    let dir = Path::new(dir);
    if !dir.is_dir() {
        tracing::warn!(dir = %dir.display(), "không thấy thư mục seed, bỏ qua nạp dữ liệu");
        return Ok(0);
    }

    let mut files: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("không đọc được thư mục {}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();

    let mut count = 0;
    for file in files {
        let raw = std::fs::read_to_string(&file).with_context(|| format!("không đọc được {}", file.display()))?;
        let foods: Vec<SeedFood> =
            serde_json::from_str(raw.trim_start_matches('\u{feff}')).with_context(|| format!("JSON không hợp lệ: {}", file.display()))?;
        for food in foods {
            food.validate().with_context(|| format!("tệp {}", file.display()))?;
            upsert(pool, &food).await.with_context(|| format!("không ghi được món '{}'", food.slug))?;
            count += 1;
        }
    }

    // Thư mục cha của seed/foods chứa seed/prices và seed/i18n/<locale>.
    let root = dir.parent().unwrap_or(dir);
    let prices = seed_prices(pool, &root.join("prices")).await?;
    let translations = seed_translations(pool, &root.join("i18n")).await?;
    refresh_search_text(pool).await?;
    tracing::info!(prices, translations, "đã nạp giá theo thị trường và bản dịch");
    Ok(count)
}

fn json_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    Ok(files)
}

fn read_json<T: serde::de::DeserializeOwned>(file: &Path) -> Result<T> {
    let raw = std::fs::read_to_string(file).with_context(|| format!("không đọc được {}", file.display()))?;
    serde_json::from_str(raw.trim_start_matches('\u{feff}')).with_context(|| format!("JSON không hợp lệ: {}", file.display()))
}

#[derive(Debug, Deserialize)]
struct SeedRange {
    min: f64,
    max: f64,
}

#[derive(Debug, Deserialize)]
struct SeedPrices {
    slug: String,
    prices: std::collections::HashMap<String, Option<SeedRange>>,
}

/// Giá theo thị trường từ `seed/prices/*.json`. Giá VN lấy từ chính tệp món ăn.
async fn seed_prices(pool: &PgPool, dir: &Path) -> Result<usize> {
    sqlx::query(
        "INSERT INTO food_prices (food_id, market, currency, price_min, price_max) \
         SELECT id, 'VN', 'VND', price_min, price_max FROM foods \
         ON CONFLICT (food_id, market) DO UPDATE SET price_min = EXCLUDED.price_min, price_max = EXCLUDED.price_max",
    )
    .execute(pool)
    .await?;

    let mut count = 0;
    for file in json_files(dir)? {
        for entry in read_json::<Vec<SeedPrices>>(&file)? {
            for (code, range) in entry.prices {
                let market = crate::i18n::market(&code).with_context(|| format!("{}: thị trường '{code}' không hỗ trợ", file.display()))?;
                match range {
                    Some(r) if r.min >= 0.0 && r.max >= r.min => {
                        sqlx::query(
                            "INSERT INTO food_prices (food_id, market, currency, price_min, price_max) \
                             SELECT id, $2, $3, $4, $5 FROM foods WHERE slug = $1 \
                             ON CONFLICT (food_id, market) DO UPDATE SET currency = EXCLUDED.currency, \
                                 price_min = EXCLUDED.price_min, price_max = EXCLUDED.price_max",
                        )
                        .bind(&entry.slug)
                        .bind(market.code)
                        .bind(market.currency)
                        .bind(r.min)
                        .bind(r.max)
                        .execute(pool)
                        .await?;
                        count += 1;
                    }
                    Some(_) => bail!("{}: giá của '{}' tại {code} không hợp lệ", file.display(), entry.slug),
                    None => {
                        sqlx::query("DELETE FROM food_prices p USING foods f WHERE p.food_id = f.id AND f.slug = $1 AND p.market = $2")
                            .bind(&entry.slug)
                            .bind(market.code)
                            .execute(pool)
                            .await?;
                    }
                }
            }
        }
    }
    Ok(count)
}

#[derive(Debug, Deserialize)]
struct SeedTranslation {
    slug: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    origin: String,
    #[serde(default)]
    cooking_style: String,
    #[serde(default)]
    serving_note: String,
    #[serde(default)]
    ingredients: Vec<SeedIngredient>,
    #[serde(default)]
    steps: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// Bản dịch nội dung món từ `seed/i18n/<locale>/*.json`.
async fn seed_translations(pool: &PgPool, dir: &Path) -> Result<usize> {
    let mut count = 0;
    for locale in crate::i18n::LOCALES.iter().filter(|l| **l != crate::i18n::DEFAULT_LOCALE) {
        for file in json_files(&dir.join(locale))? {
            for t in read_json::<Vec<SeedTranslation>>(&file)? {
                let ingredients = json!(t.ingredients.iter().map(|i| json!({ "name": i.name, "amount": i.amount })).collect::<Vec<_>>());
                let affected = sqlx::query(
                    "INSERT INTO food_translations (food_id, locale, name, description, origin, cooking_style, serving_note, ingredients, steps, tags) \
                     SELECT id, $2, $3, $4, $5, $6, $7, $8, $9, $10 FROM foods WHERE slug = $1 \
                     ON CONFLICT (food_id, locale) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description, \
                         origin = EXCLUDED.origin, cooking_style = EXCLUDED.cooking_style, serving_note = EXCLUDED.serving_note, \
                         ingredients = EXCLUDED.ingredients, steps = EXCLUDED.steps, tags = EXCLUDED.tags",
                )
                .bind(&t.slug)
                .bind(locale)
                .bind(&t.name)
                .bind(&t.description)
                .bind(&t.origin)
                .bind(&t.cooking_style)
                .bind(&t.serving_note)
                .bind(&ingredients)
                .bind(json!(t.steps))
                .bind(&t.tags)
                .execute(pool)
                .await?
                .rows_affected();
                if affected == 0 {
                    tracing::warn!(slug = %t.slug, locale, "bản dịch cho món không tồn tại, bỏ qua");
                }
                count += affected as usize;
            }
        }
    }
    Ok(count)
}

/// (id, tên, tên gốc, thẻ, nền ẩm thực, tên và thẻ ở các ngôn ngữ khác)
type SearchRow = (uuid::Uuid, String, Option<String>, Vec<String>, String, Option<String>);

/// Gộp tên món ở mọi ngôn ngữ vào cột tìm kiếm, để gõ "ramen", "ラーメン" hay "라멘" đều ra.
async fn refresh_search_text(pool: &PgPool) -> Result<()> {
    let rows: Vec<SearchRow> = sqlx::query_as(
        "SELECT f.id, f.name, f.original_name, f.tags, f.cuisine, string_agg(t.name || ' ' || array_to_string(t.tags, ' '), ' ') \
         FROM foods f LEFT JOIN food_translations t ON t.food_id = f.id GROUP BY f.id",
    )
    .fetch_all(pool)
    .await?;
    for (id, name, original, tags, cuisine, translated) in rows {
        let text = fold_vi(&format!(
            "{name} {} {} {} {}",
            original.unwrap_or_default(),
            tags.join(" "),
            cuisine_label_vi(&cuisine),
            translated.unwrap_or_default()
        ));
        sqlx::query("UPDATE foods SET search_text = $1 WHERE id = $2").bind(text).bind(id).execute(pool).await?;
    }
    Ok(())
}

async fn upsert(pool: &PgPool, food: &SeedFood) -> Result<()> {
    let search_text = fold_vi(&format!(
        "{} {} {} {}",
        food.name,
        food.original_name.as_deref().unwrap_or(""),
        food.tags.join(" "),
        cuisine_label_vi(&food.cuisine)
    ));
    let ingredients = json!(food.ingredients.iter().map(|i| json!({ "name": i.name, "amount": i.amount })).collect::<Vec<_>>());

    sqlx::query(
        "INSERT INTO foods (slug, name, original_name, cuisine, style, dish_type, meals, soup_type, price_min, price_max, \
             calories_kcal, serving_note, protein_g, carbs_g, fat_g, description, origin, cooking_style, prep_minutes, \
             cook_minutes, servings, ingredients, steps, tags, image_url, image_source_url, image_author, image_license, \
             featured, weight, search_text) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, \
             $24, $25, $26, $27, $28, $29, $30, $31) \
         ON CONFLICT (slug) DO UPDATE SET \
             name = EXCLUDED.name, original_name = EXCLUDED.original_name, cuisine = EXCLUDED.cuisine, style = EXCLUDED.style, \
             dish_type = EXCLUDED.dish_type, meals = EXCLUDED.meals, soup_type = EXCLUDED.soup_type, \
             price_min = EXCLUDED.price_min, price_max = EXCLUDED.price_max, calories_kcal = EXCLUDED.calories_kcal, \
             serving_note = EXCLUDED.serving_note, protein_g = EXCLUDED.protein_g, carbs_g = EXCLUDED.carbs_g, \
             fat_g = EXCLUDED.fat_g, description = EXCLUDED.description, origin = EXCLUDED.origin, \
             cooking_style = EXCLUDED.cooking_style, prep_minutes = EXCLUDED.prep_minutes, cook_minutes = EXCLUDED.cook_minutes, \
             servings = EXCLUDED.servings, ingredients = EXCLUDED.ingredients, steps = EXCLUDED.steps, tags = EXCLUDED.tags, \
             image_url = EXCLUDED.image_url, image_source_url = EXCLUDED.image_source_url, image_author = EXCLUDED.image_author, \
             image_license = EXCLUDED.image_license, featured = EXCLUDED.featured, weight = EXCLUDED.weight, \
             search_text = EXCLUDED.search_text, updated_at = now()",
    )
    .bind(&food.slug)
    .bind(&food.name)
    .bind(&food.original_name)
    .bind(&food.cuisine)
    .bind(&food.style)
    .bind(&food.dish_type)
    .bind(&food.meals)
    .bind(&food.soup_type)
    .bind(food.price_min)
    .bind(food.price_max)
    .bind(food.calories_kcal)
    .bind(&food.serving_note)
    .bind(food.nutrition.protein_g)
    .bind(food.nutrition.carbs_g)
    .bind(food.nutrition.fat_g)
    .bind(&food.description)
    .bind(&food.origin)
    .bind(&food.cooking_style)
    .bind(food.prep_minutes)
    .bind(food.cook_minutes)
    .bind(food.servings.max(1))
    .bind(&ingredients)
    .bind(json!(food.steps))
    .bind(&food.tags)
    .bind(format!("/images/foods/{}", food.image.file))
    .bind(&food.image.source_url)
    .bind(&food.image.author)
    .bind(&food.image.license)
    .bind(food.featured)
    .bind(food.weight.filter(|w| w.is_finite() && *w > 0.0).unwrap_or(1.0))
    .bind(search_text)
    .execute(pool)
    .await?;
    Ok(())
}
