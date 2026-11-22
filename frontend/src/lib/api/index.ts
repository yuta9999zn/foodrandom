import type { Locale, MarketCode } from '@/i18n/config';

import { api, ApiError } from './client';
import type { FilterMeta, FoodDetail, FoodFilter, FoodListResponse, GeoPoint, ProviderInfo, RandomResponse, SortKey } from '../types';

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
  meta: () => api<FilterMeta>('/meta/filters'),
};

export const locationApi = {
  provider: () => api<ProviderInfo>('/location/provider'),
  geocode: (q: string, prefs: Prefs) => api<ProviderInfo & { results: GeoPoint[] }>('/location/geocode', { query: { q, ...prefs }, timeoutMs: 35_000 }),
  reverse: (lat: number, lng: number, lang: Locale) =>
    api<ProviderInfo & { label: string | null }>('/location/reverse', { query: { lat, lng, lang } }),
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
