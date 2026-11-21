import type { Locale, MarketCode } from '@/i18n/config';

import { api, ApiError } from './client';
import type {
  FavoritePlace,
  FilterMeta,
  FoodDetail,
  FoodFilter,
  FoodListResponse,
  FoodSummary,
  GeoPoint,
  NearbyResponse,
  Place,
  ProviderInfo,
  RandomResponse,
  RatingResponse,
  SortKey,
  UserProfile,
} from '../types';

export * from './client';

/** Ngôn ngữ + thị trường gửi kèm mọi request trả nội dung món ăn. */
export interface Prefs {
  lang: Locale;
  market: MarketCode;
}

export interface ListParams {
  filter: FoodFilter;
  sort: SortKey | null;
  page: number;
}

const csv = (values: readonly string[]) => (values.length ? values.join(',') : undefined);

export const foodsApi = {
  list: ({ filter, sort, page }: ListParams, prefs: Prefs, signal?: AbortSignal) =>
    api<FoodListResponse>('/foods', {
      signal,
      query: {
        ...prefs,
        meals: csv(filter.meals),
        dish_type: filter.dish_type,
        style: filter.style,
        soup_type: filter.soup_type,
        cuisines: csv(filter.cuisines),
        budgets: csv(filter.budgets),
        q: filter.q,
        lat: filter.lat,
        lng: filter.lng,
        radius_km: filter.radius_km,
        sort,
        page,
      },
    }),
  detail: (slug: string, prefs: Prefs, init?: RequestInit) =>
    api<FoodDetail>(`/foods/${encodeURIComponent(slug)}`, { ...init, query: { ...prefs } }),
  random: (
    body: { filter: Partial<FoodFilter>; sort?: SortKey | null; page?: number; scope?: 'page' | 'all'; exclude?: string[] },
    prefs: Prefs,
  ) => api<RandomResponse>('/random', { method: 'POST', json: { ...body, filter: { ...body.filter, ...prefs } } }),
  rating: (slug: string) => api<RatingResponse>(`/foods/${encodeURIComponent(slug)}/rating`),
  rate: (slug: string, score: number) =>
    api<RatingResponse>(`/foods/${encodeURIComponent(slug)}/rating`, { method: 'PUT', json: { score } }),
  meta: () => api<FilterMeta>('/meta/filters'),
};

export const locationApi = {
  provider: () => api<ProviderInfo>('/location/provider'),
  nearby: (
    params: { lat: number; lng: number; radius_km?: number | null; food?: string; cuisine?: string; limit?: number; sort?: 'rating' | 'distance' },
    prefs: Prefs,
  ) =>
    api<NearbyResponse>('/location/nearby', { query: { ...params, ...prefs }, timeoutMs: 35_000 }),
  geocode: (q: string, prefs: Prefs) => api<ProviderInfo & { results: GeoPoint[] }>('/location/geocode', { query: { q, ...prefs }, timeoutMs: 35_000 }),
  reverse: (lat: number, lng: number, lang: Locale) =>
    api<ProviderInfo & { label: string | null }>('/location/reverse', { query: { lat, lng, lang } }),
};

export interface OtpSent {
  message: string;
  expires_in: number;
  resend_after: number;
}

export interface AuthResult {
  user: UserProfile;
  message: string;
}

export const authApi = {
  me: () => api<UserProfile>('/auth/me'),
  requestRegisterOtp: (body: { email: string; phone: string }) =>
    api<OtpSent>('/auth/register/request-otp', { method: 'POST', json: body }),
  verifyRegister: (body: { email: string; otp: string }) => api<AuthResult>('/auth/register/verify', { method: 'POST', json: body }),
  login: (body: { identifier: string; password: string }) => api<AuthResult>('/auth/login', { method: 'POST', json: body }),
  logout: () => api<{ message: string }>('/auth/logout', { method: 'POST' }),
  forgotPassword: (body: { email: string }) => api<OtpSent>('/auth/password/forgot', { method: 'POST', json: body }),
  resetPassword: (body: { email: string; otp: string }) => api<{ message: string }>('/auth/password/reset', { method: 'POST', json: body }),
  changePassword: (body: { current_password: string; new_password: string }) =>
    api<{ message: string }>('/auth/password/change', { method: 'POST', json: body }),
  updateProfile: (body: { display_name: string }) => api<UserProfile>('/auth/me', { method: 'PATCH', json: body }),
};

export const favoritesApi = {
  foods: (prefs: Prefs) => api<{ items: FoodSummary[] }>('/favorites/foods', { query: { ...prefs } }),
  foodSlugs: () => api<{ slugs: string[] }>('/favorites/foods/slugs'),
  addFood: (slug: string) => api<{ slug: string; favorited: boolean }>(`/favorites/foods/${encodeURIComponent(slug)}`, { method: 'PUT' }),
  removeFood: (slug: string) =>
    api<{ slug: string; favorited: boolean }>(`/favorites/foods/${encodeURIComponent(slug)}`, { method: 'DELETE' }),
  places: () => api<{ items: FavoritePlace[] }>('/favorites/places'),
  savePlace: (place: Place, extra: { cuisine?: string | null; food_slug?: string | null }) =>
    api<FavoritePlace>('/favorites/places', {
      method: 'POST',
      json: {
        provider: place.provider,
        place_ref: place.place_ref,
        name: place.name,
        address: place.address,
        lat: place.lat,
        lng: place.lng,
        maps_url: place.maps_url,
        cuisine: extra.cuisine ?? place.cuisines[0] ?? null,
        food_slug: extra.food_slug ?? null,
      },
    }),
  removePlace: (id: string) => api<void>(`/favorites/places/${id}`, { method: 'DELETE' }),
};

type T = (key: string, vars?: Record<string, string | number>) => string;

/**
 * Thông báo lỗi theo ngôn ngữ người dùng. Tiếng Việt dùng nguyên thông báo chi tiết của backend;
 * ngôn ngữ khác dùng thông báo theo mã lỗi (backend hiện chỉ trả tiếng Việt).
 */
export function localizedError(error: unknown, t: T, locale: Locale): string {
  if (!(error instanceof ApiError)) return t('errors.generic');
  if (locale === 'vi' && error.status !== 0) return error.message;
  const key = `errors.${error.code}`;
  const message = t(key);
  return message === key ? t('errors.generic') : message;
}
