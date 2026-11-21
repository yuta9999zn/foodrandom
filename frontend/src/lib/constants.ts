import type { Cuisine, DishType, Meal, SortKey, SoupType, Style } from './types';

export const PAGE_SIZE = 12;

// Chỉ giữ giá trị; nhãn hiển thị lấy từ từ điển i18n (options.*).
export const MEALS: Meal[] = ['breakfast', 'lunch', 'dinner', 'snack'];
export const DISH_TYPES: DishType[] = ['main', 'side', 'fast_food', 'snack', 'dessert', 'drink'];
export const STYLES: Style[] = ['traditional', 'modern'];
export const SOUP_TYPES: SoupType[] = ['pho', 'bun', 'mi', 'hu_tieu', 'banh_canh', 'lau', 'kho'];
export const CUISINES: Cuisine[] = ['vietnam', 'japan', 'china', 'korea', 'uk'];
export const SORTS: SortKey[] = ['nearest', 'featured', 'price_asc', 'price_desc', 'rating'];

/** Món nước = mọi nhóm trừ "Món khô". */
export const isSoup = (value: SoupType) => value !== 'kho';
