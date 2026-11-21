use std::collections::HashMap;

use rand::distr::{weighted::WeightedIndex, Distribution};
use uuid::Uuid;

use super::{
    dto::{
        FilterMeta, FoodDetail, FoodFilter, FoodListItem, FoodListResponse, FoodSummary, ImageCredit,
        Ingredient, LocationStatus, Nutrition, OptionCount, Paging, RandomRequest, RandomResponse, RatingResponse,
        CUISINES,
    },
    repository::FoodRepository,
};
use crate::{
    error::{AppError, AppResult},
    i18n::{self, Market},
    infra::cache::Cache,
    modules::location::{dto::Place, provider::cuisine_label_vi, service::LocationService},
    util::fold_vi,
};

const DETAIL_CACHE_TTL_SECS: u64 = 300;

/// Tập kết quả của một bộ lọc cho một người dùng: phần khớp hoàn toàn (đã sắp xếp)
/// và phần gợi ý mở rộng dùng để lấp trang cho đủ 12 món.
struct Catalog {
    exact: Vec<FoodListItem>,
    relaxed: Vec<FoodListItem>,
    sort: String,
    location: LocationStatus,
}

struct Page {
    items: Vec<FoodListItem>,
    total: usize,
    total_pages: usize,
    relaxed_count: usize,
}

#[derive(Clone)]
pub struct FoodService {
    repo: FoodRepository,
    cache: Cache,
    location: LocationService,
    cache_ttl_secs: u64,
}

impl FoodService {
    pub fn new(repo: FoodRepository, cache: Cache, location: LocationService, cache_ttl_secs: u64) -> Self {
        Self { repo, cache, location, cache_ttl_secs }
    }

    /// Tập món khớp bộ lọc, có cache Redis ngắn hạn. Không phụ thuộc người dùng nên dùng chung được.
    async fn search_cached(&self, filter: &FoodFilter) -> AppResult<Vec<FoodSummary>> {
        let key = format!("foods:search:{}", filter.catalog_key());
        if let Some(hit) = self.cache.get_json::<Vec<FoodSummary>>(&key).await {
            return Ok(hit);
        }
        let foods = self.repo.search(filter).await?;
        self.cache.set_json(&key, &foods, self.cache_ttl_secs).await;
        Ok(foods)
    }

    async fn catalog(&self, filter: &FoodFilter, requested_sort: Option<&str>) -> AppResult<Catalog> {
        let foods = self.search_cached(filter).await?;

        // Vị trí: gắn quán gần nhất theo nền ẩm thực. Nhà cung cấp lỗi thì vẫn trả danh sách, chỉ thiếu khoảng cách.
        let (nearest, location) = match filter.coords() {
            None => (None, LocationStatus { status: "disabled".into(), source: None, message: None }),
            // Không chờ nhà cung cấp địa điểm: có sẵn trong cache thì gắn ngay, chưa có thì tìm ở nền
            // và trả `pending` để client hỏi lại sau vài giây. Danh sách món không bao giờ bị chậm vì bản đồ.
            Some((lat, lng)) => match self.location.nearest_by_cuisine_cached(lat, lng, filter.radius_km, i18n::parse_market(Some(&filter.market))?).await {
                Ok(Some(map)) => (
                    Some(map),
                    LocationStatus { status: "ok".into(), source: Some(self.location.info()), message: None },
                ),
                Ok(None) => (
                    None,
                    LocationStatus { status: "pending".into(), source: Some(self.location.info()), message: None },
                ),
                Err(err) => (
                    None,
                    LocationStatus {
                        status: "unavailable".into(),
                        source: Some(self.location.info()),
                        message: Some(err.to_string()),
                    },
                ),
            },
        };

        let attach = |food: FoodSummary, level: &str, nearest: &Option<HashMap<String, Place>>| FoodListItem {
            nearest_place: nearest.as_ref().and_then(|m| m.get(&food.cuisine).cloned()),
            match_level: level.to_string(),
            food,
        };

        let mut exact: Vec<FoodListItem> = foods.into_iter().map(|f| attach(f, "exact", &nearest)).collect();

        // Có bán kính và có dữ liệu địa điểm: chỉ giữ món có quán trong bán kính.
        let radius_applied = filter.radius_km.is_some() && nearest.is_some();
        if radius_applied {
            exact.retain(|item| item.nearest_place.is_some());
        }

        let sort = match requested_sort {
            Some("nearest") if nearest.is_none() => "featured",
            Some(sort) => sort,
            None if nearest.is_some() => "nearest",
            None => "featured",
        }
        .to_string();
        sort_items(&mut exact, &sort);

        // Gợi ý mở rộng: các món còn lại, xếp theo số tiêu chí khớp giảm dần.
        let relaxed = if filter.has_criteria() || radius_applied {
            let base = FoodFilter { market: filter.market.clone(), lang: filter.lang.clone(), ..Default::default() };
            let everything = self.search_cached(&base).await?;
            let mut scored: Vec<(usize, FoodListItem)> = everything
                .into_iter()
                .filter(|f| !exact.iter().any(|e| e.food.id == f.id))
                // Món không bán ở thị trường này không dùng để lấp trang.
                .filter(|f| f.price_min.is_some())
                .map(|f| (match_score(filter, &f), attach(f, "relaxed", &nearest)))
                .collect();
            scored.sort_by(|(sa, a), (sb, b)| {
                sb.cmp(sa)
                    .then_with(|| b.nearest_place.is_some().cmp(&a.nearest_place.is_some()))
                    .then_with(|| b.food.featured.cmp(&a.food.featured))
                    .then_with(|| a.food.name.cmp(&b.food.name))
            });
            scored.into_iter().map(|(_, item)| item).collect()
        } else {
            Vec::new()
        };

        Ok(Catalog { exact, relaxed, sort, location })
    }

    fn page(catalog: &Catalog, page: usize, page_size: usize) -> Page {
        let total = catalog.exact.len();
        let total_pages = total.div_ceil(page_size).max(1);
        let mut items: Vec<FoodListItem> =
            catalog.exact.iter().skip((page - 1) * page_size).take(page_size).cloned().collect();
        let mut relaxed_count = 0;
        // Trang hợp lệ nhưng chưa đủ món: lấp bằng gợi ý mở rộng để luôn có đủ `page_size` món.
        if page <= total_pages && items.len() < page_size {
            let missing = page_size - items.len();
            let fill: Vec<FoodListItem> = catalog.relaxed.iter().take(missing).cloned().collect();
            relaxed_count = fill.len();
            items.extend(fill);
        }
        Page { items, total, total_pages, relaxed_count }
    }

    pub async fn list(&self, filter: FoodFilter, paging: Paging) -> AppResult<FoodListResponse> {
        let catalog = self.catalog(&filter, paging.sort.as_deref()).await?;
        let page = Self::page(&catalog, paging.page, paging.page_size);
        Ok(FoodListResponse {
            items: page.items,
            page: paging.page,
            page_size: paging.page_size,
            total: page.total,
            total_pages: page.total_pages,
            relaxed_count: page.relaxed_count,
            sort: catalog.sort,
            currency: i18n::market(&filter.market).map(|m| m.currency).unwrap_or("VND").to_string(),
            market: filter.market,
            lang: filter.lang,
            location: catalog.location,
        })
    }

    /// Random Engine. Tập ứng viên được dựng lại ở backend từ bộ lọc, nên frontend không thể
    /// đưa vào một món nằm ngoài kết quả lọc.
    pub async fn random(&self, request: RandomRequest) -> AppResult<RandomResponse> {
        let filter = request.filter.normalize()?;
        let paging = Paging::new(request.sort, request.page, request.page_size)?;
        let scope = match request.scope.as_deref().unwrap_or("page") {
            "page" => "page",
            "all" => "all",
            other => return Err(AppError::validation("scope", format!("Giá trị '{other}' không hợp lệ. Cho phép: page, all"))),
        };

        let catalog = self.catalog(&filter, paging.sort.as_deref()).await?;
        let candidates: Vec<FoodListItem> = if scope == "all" {
            catalog.exact.clone()
        } else {
            Self::page(&catalog, paging.page, paging.page_size).items
        };

        // Ưu tiên món khớp hoàn toàn bộ lọc; chỉ dùng gợi ý mở rộng khi không có món nào khớp.
        let mut pool: Vec<FoodListItem> = candidates.iter().filter(|i| i.match_level == "exact").cloned().collect();
        if pool.is_empty() {
            pool = if scope == "all" { catalog.relaxed.iter().take(paging.page_size).cloned().collect() } else { candidates };
        }
        if pool.is_empty() {
            return Err(AppError::NotFound("Không có món nào phù hợp để chọn ngẫu nhiên. Hãy nới bộ lọc.".into()));
        }
        let pool_size = pool.len();

        // Tránh lặp lại món vừa chọn, trừ khi loại hết thì không còn gì để chọn.
        let fresh: Vec<FoodListItem> = pool.iter().filter(|i| !request.exclude.contains(&i.food.slug)).cloned().collect();
        let pool = if fresh.is_empty() { pool } else { fresh };

        let picked = pick_weighted(&pool);
        Ok(RandomResponse { food: picked, pool_size, scope: scope.to_string(), page: paging.page })
    }

    pub async fn detail(&self, slug: &str, lang: &str, market: &Market) -> AppResult<FoodDetail> {
        let key = format!("foods:detail:{slug}:{lang}:{}", market.code);
        if let Some(hit) = self.cache.get_json::<FoodDetail>(&key).await {
            return Ok(hit);
        }
        let row = self
            .repo
            .find_by_slug(slug, market.code, lang)
            .await?
            .ok_or_else(|| AppError::NotFound("Không tìm thấy món ăn".into()))?;
        let similar = self.repo.similar(&row.summary, market.code, lang, 8).await?;
        let detail = FoodDetail {
            origin: row.origin,
            cooking_style: row.cooking_style,
            prep_minutes: row.prep_minutes,
            cook_minutes: row.cook_minutes,
            servings: row.servings,
            nutrition: Nutrition {
                calories_kcal: row.summary.calories_kcal,
                protein_g: row.protein_g,
                carbs_g: row.carbs_g,
                fat_g: row.fat_g,
                serving_note: row.serving_note,
            },
            ingredients: serde_json::from_value::<Vec<Ingredient>>(row.ingredients).unwrap_or_default(),
            steps: serde_json::from_value::<Vec<String>>(row.steps).unwrap_or_default(),
            tags: row.tags,
            image_credit: ImageCredit {
                source_url: row.image_source_url,
                author: row.image_author,
                license: row.image_license,
            },
            similar,
            food: row.summary,
        };
        self.cache.set_json(&key, &detail, DETAIL_CACHE_TTL_SECS).await;
        Ok(detail)
    }

    async fn food_id(&self, slug: &str) -> AppResult<Uuid> {
        self.repo.id_by_slug(slug).await?.ok_or_else(|| AppError::NotFound("Không tìm thấy món ăn".into()))
    }

    pub async fn rate(&self, user_id: Uuid, slug: &str, score: i16) -> AppResult<RatingResponse> {
        let food_id = self.food_id(slug).await?;
        self.repo.upsert_rating(user_id, food_id, score).await?;
        let keys: Vec<String> = i18n::LOCALES
            .iter()
            .flat_map(|l| i18n::MARKETS.iter().map(move |m| format!("foods:detail:{slug}:{l}:{}", m.code)))
            .collect();
        let refs: Vec<&str> = keys.iter().map(String::as_str).collect();
        let _ = self.cache.delete(&refs).await;
        let (rating_avg, rating_count) = self.repo.rating_summary(food_id).await?;
        Ok(RatingResponse { rating_avg, rating_count, mine: Some(score) })
    }

    pub async fn rating(&self, user_id: Option<Uuid>, slug: &str) -> AppResult<RatingResponse> {
        let food_id = self.food_id(slug).await?;
        let (rating_avg, rating_count) = self.repo.rating_summary(food_id).await?;
        let mine = match user_id {
            Some(user_id) => self.repo.user_rating(user_id, food_id).await?,
            None => None,
        };
        Ok(RatingResponse { rating_avg, rating_count, mine })
    }

    pub async fn meta(&self) -> AppResult<FilterMeta> {
        let counts: HashMap<String, i64> = self.repo.count_by_cuisine().await?.into_iter().collect();
        let cuisines: Vec<OptionCount> = CUISINES
            .iter()
            .map(|code| OptionCount {
                code: code.to_string(),
                label: cuisine_label_vi(code).to_string(),
                count: counts.get(*code).copied().unwrap_or(0),
            })
            .collect();
        Ok(FilterMeta {
            total_foods: cuisines.iter().map(|c| c.count).sum(),
            cuisines,
            page_size: super::dto::PAGE_SIZE,
            markets: i18n::MARKETS.to_vec(),
            locales: i18n::LOCALES.to_vec(),
        })
    }
}

fn distance_of(item: &FoodListItem) -> f64 {
    item.nearest_place.as_ref().map(|p| p.distance_km).unwrap_or(f64::INFINITY)
}

fn sort_items(items: &mut [FoodListItem], sort: &str) {
    match sort {
        "nearest" => items.sort_by(|a, b| {
            distance_of(a)
                .total_cmp(&distance_of(b))
                .then_with(|| b.food.featured.cmp(&a.food.featured))
                .then_with(|| a.food.name.cmp(&b.food.name))
        }),
        // Món không có giá ở thị trường này luôn xếp cuối.
        "price_asc" => items.sort_by(|a, b| {
            let key = |i: &FoodListItem| (i.food.price_min.unwrap_or(f64::INFINITY), i.food.price_max.unwrap_or(f64::INFINITY));
            let (ka, kb) = (key(a), key(b));
            ka.0.total_cmp(&kb.0).then(ka.1.total_cmp(&kb.1)).then_with(|| a.food.name.cmp(&b.food.name))
        }),
        "price_desc" => items.sort_by(|a, b| {
            let key = |i: &FoodListItem| (i.food.price_max.unwrap_or(f64::NEG_INFINITY), i.food.price_min.unwrap_or(f64::NEG_INFINITY));
            let (ka, kb) = (key(a), key(b));
            kb.0.total_cmp(&ka.0).then(kb.1.total_cmp(&ka.1)).then_with(|| a.food.name.cmp(&b.food.name))
        }),
        "rating" => items.sort_by(|a, b| {
            let (ra, rb) = (a.food.rating_avg.unwrap_or(-1.0), b.food.rating_avg.unwrap_or(-1.0));
            rb.total_cmp(&ra)
                .then_with(|| b.food.rating_count.cmp(&a.food.rating_count))
                .then_with(|| b.food.featured.cmp(&a.food.featured))
                .then_with(|| a.food.name.cmp(&b.food.name))
        }),
        "popular" => items.sort_by(|a, b| {
            b.food
                .favorite_count
                .cmp(&a.food.favorite_count)
                .then_with(|| b.food.rating_count.cmp(&a.food.rating_count))
                .then_with(|| b.food.featured.cmp(&a.food.featured))
                .then_with(|| a.food.name.cmp(&b.food.name))
        }),
        // "featured": giữ thứ tự của repository (nổi bật trước, rồi theo tên).
        _ => {}
    }
}

fn in_budget(food: &FoodSummary, lo: f64, hi: Option<f64>) -> bool {
    let (Some(min), Some(max)) = (food.price_min, food.price_max) else { return false };
    max.max(min + 0.01) > lo && hi.is_none_or(|hi| min < hi)
}

/// Số tiêu chí của bộ lọc mà món thoả. Dùng để xếp hạng gợi ý mở rộng.
fn match_score(filter: &FoodFilter, food: &FoodSummary) -> usize {
    let mut score = 0;
    if !filter.meals.is_empty() && filter.meals.iter().any(|m| food.meals.contains(m)) {
        score += 1;
    }
    if filter.dish_type.as_deref() == Some(food.dish_type.as_str()) {
        score += 1;
    }
    if filter.style.as_deref() == Some(food.style.as_str()) {
        score += 1;
    }
    if filter.soup_type.as_deref() == Some(food.soup_type.as_str()) {
        score += 1;
    }
    if filter.cuisines.contains(&food.cuisine) {
        score += 1;
    }
    let market = i18n::market(&filter.market);
    if filter.budgets.iter().filter_map(|c| market.and_then(|m| m.band(c))).any(|b| in_budget(food, b.min, b.max)) {
        score += 1;
    }
    if filter.price_min.is_some() || filter.price_max.is_some() {
        let lo_ok = filter.price_min.is_none_or(|lo| food.price_max.is_some_and(|max| max >= lo));
        let hi_ok = filter.price_max.is_none_or(|hi| food.price_min.is_some_and(|min| min <= hi));
        if lo_ok && hi_ok {
            score += 1;
        }
    }
    if let Some(q) = &filter.q {
        if fold_vi(&food.name).contains(&fold_vi(q)) {
            score += 1;
        }
    }
    score
}

/// Chọn ngẫu nhiên có trọng số theo cột `weight` của món (mặc định 1.0 = xác suất đều).
fn pick_weighted(pool: &[FoodListItem]) -> FoodListItem {
    let mut rng = rand::rng();
    let weights: Vec<f32> = pool.iter().map(|i| if i.food.weight.is_finite() && i.food.weight > 0.0 { i.food.weight } else { 1.0 }).collect();
    let index = match WeightedIndex::new(&weights) {
        Ok(dist) => dist.sample(&mut rng),
        Err(_) => 0,
    };
    pool[index].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn food(slug: &str, cuisine: &str, price: (i32, i32), weight: f32) -> FoodSummary {
        let price = (f64::from(price.0), f64::from(price.1));
        FoodSummary {
            id: Uuid::new_v4(),
            slug: slug.into(),
            name: slug.into(),
            original_name: None,
            cuisine: cuisine.into(),
            style: "traditional".into(),
            dish_type: "main".into(),
            meals: vec!["lunch".into()],
            soup_type: "kho".into(),
            price_min: Some(price.0),
            price_max: Some(price.1),
            currency: Some("VND".into()),
            calories_kcal: 500,
            description: String::new(),
            image_url: String::new(),
            featured: false,
            weight,
            rating_avg: None,
            rating_count: 0,
            favorite_count: 0,
        }
    }

    fn item(food: FoodSummary, level: &str) -> FoodListItem {
        FoodListItem { food, match_level: level.into(), nearest_place: None }
    }

    fn catalog(exact: usize, relaxed: usize) -> Catalog {
        Catalog {
            exact: (0..exact).map(|i| item(food(&format!("e{i}"), "vietnam", (30_000, 50_000), 1.0), "exact")).collect(),
            relaxed: (0..relaxed).map(|i| item(food(&format!("r{i}"), "japan", (30_000, 50_000), 1.0), "relaxed")).collect(),
            sort: "featured".into(),
            location: LocationStatus { status: "disabled".into(), source: None, message: None },
        }
    }

    #[test]
    fn budget_bands_do_not_leak_across_boundaries() {
        let pho = food("pho", "vietnam", (35_000, 50_000), 1.0);
        assert!(in_budget(&pho, 30_000.0, Some(50_000.0)));
        assert!(!in_budget(&pho, 50_000.0, Some(100_000.0)));
        assert!(!in_budget(&pho, 0.0, Some(30_000.0)));
        let flat = food("flat", "vietnam", (30_000, 30_000), 1.0);
        assert!(in_budget(&flat, 30_000.0, Some(50_000.0)));
        let pricey = food("wagyu", "japan", (250_000, 900_000), 1.0);
        assert!(in_budget(&pricey, 200_000.0, None));
        let unavailable = FoodSummary { price_min: None, price_max: None, ..pho.clone() };
        assert!(!in_budget(&unavailable, 0.0, None));
    }

    #[test]
    fn page_is_filled_to_twelve_with_relaxed_suggestions() {
        let c = catalog(5, 30);
        let p = FoodService::page(&c, 1, 12);
        assert_eq!(p.items.len(), 12);
        assert_eq!((p.total, p.total_pages, p.relaxed_count), (5, 1, 7));
        assert!(p.items[..5].iter().all(|i| i.match_level == "exact"));
    }

    #[test]
    fn second_page_keeps_exact_items_first() {
        let c = catalog(14, 30);
        let p1 = FoodService::page(&c, 1, 12);
        let p2 = FoodService::page(&c, 2, 12);
        assert_eq!((p1.items.len(), p1.relaxed_count), (12, 0));
        assert_eq!((p2.items.len(), p2.relaxed_count, p2.total_pages), (12, 10, 2));
        assert_eq!(p2.items[0].food.slug, "e12");
        assert!(FoodService::page(&c, 3, 12).items.is_empty());
    }

    #[test]
    fn weighted_pick_stays_inside_pool_and_respects_weight() {
        let pool = vec![
            item(food("a", "vietnam", (1, 2), 1.0), "exact"),
            item(food("b", "vietnam", (1, 2), 1000.0), "exact"),
        ];
        let picks_b = (0..200).filter(|_| pick_weighted(&pool).food.slug == "b").count();
        assert!(picks_b > 180, "món trọng số cao phải được chọn áp đảo, thực tế {picks_b}/200");
    }

    #[test]
    fn match_score_counts_satisfied_criteria() {
        let filter = FoodFilter {
            market: "VN".into(),
            cuisines: vec!["vietnam".into()],
            budgets: vec!["b2".into()],
            style: Some("modern".into()),
            ..Default::default()
        };
        assert_eq!(match_score(&filter, &food("x", "vietnam", (35_000, 45_000), 1.0)), 2);
        assert_eq!(match_score(&filter, &food("y", "japan", (300_000, 400_000), 1.0)), 0);
    }
}
