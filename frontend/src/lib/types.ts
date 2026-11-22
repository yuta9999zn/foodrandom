// Kiểu dữ liệu khớp với DTO của backend (xem /docs của API).

export type Cuisine = 'vietnam' | 'japan' | 'china' | 'korea' | 'uk';
export type Style = 'traditional' | 'modern';
export type DishType = 'main' | 'side' | 'fast_food' | 'snack' | 'dessert' | 'drink';
export type Meal = 'breakfast' | 'lunch' | 'dinner' | 'snack';
export type SoupType = 'pho' | 'bun' | 'mi' | 'hu_tieu' | 'banh_canh' | 'lau' | 'kho' | 'khac';
export type Budget = 'b1' | 'b2' | 'b3' | 'b4' | 'b5';
export type SortKey = 'featured' | 'nearest' | 'price_asc' | 'price_desc' | 'rating' | 'popular';

export interface Place {
  provider: 'google' | 'osm' | 'mock';
  place_ref: string;
  name: string;
  address: string | null;
  lat: number;
  lng: number;
  distance_km: number;
  cuisines: Cuisine[];
  rating: number | null;
  rating_count: number | null;
  open_now: boolean | null;
  maps_url: string;
  is_mock: boolean;
}

export interface FoodSummary {
  id: string;
  slug: string;
  name: string;
  original_name: string | null;
  cuisine: Cuisine;
  style: Style;
  dish_type: DishType;
  meals: Meal[];
  soup_type: SoupType;
  /** Giá tại thị trường đang chọn; `null` = món ít phổ biến ở thị trường này. */
  price_min: number | null;
  price_max: number | null;
  currency: string | null;
  calories_kcal: number;
  description: string;
  image_url: string;
  featured: boolean;
  weight: number;
  rating_avg: number | null;
  rating_count: number;
  favorite_count: number;
}

export interface FoodListItem extends FoodSummary {
  match_level: 'exact' | 'relaxed';
  nearest_place: Place | null;
}

export interface ProviderInfo {
  provider: string;
  is_mock: boolean;
  attribution: string;
}

export interface LocationStatus extends Partial<ProviderInfo> {
  status: 'disabled' | 'ok' | 'pending' | 'unavailable';
  message: string | null;
}

export interface FoodListResponse {
  items: FoodListItem[];
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
  relaxed_count: number;
  sort: SortKey;
  market: string;
  currency: string;
  lang: string;
  location: LocationStatus;
}

export interface Ingredient {
  name: string;
  amount: string;
}

export interface FoodDetail extends FoodSummary {
  origin: string;
  cooking_style: string;
  prep_minutes: number;
  cook_minutes: number;
  servings: number;
  nutrition: {
    calories_kcal: number;
    protein_g: number;
    carbs_g: number;
    fat_g: number;
    serving_note: string;
  };
  ingredients: Ingredient[];
  steps: string[];
  tags: string[];
  image_credit: { source_url: string | null; author: string | null; license: string | null };
  similar: FoodSummary[];
}

export interface FoodFilter {
  meals: Meal[];
  dish_type: DishType | null;
  style: Style | null;
  soup_type: SoupType | null;
  cuisines: Cuisine[];
  budgets: Budget[];
  q?: string | null;
  lat?: number | null;
  lng?: number | null;
  radius_km?: number | null;
}

export interface RandomResponse {
  food: FoodListItem;
  pool_size: number;
  scope: 'page' | 'all';
  page: number;
}

export interface FilterMeta {
  total_foods: number;
  cuisines: { code: Cuisine; label: string; count: number }[];
  page_size: number;
}

export interface GeoPoint {
  label: string;
  lat: number;
  lng: number;
}
