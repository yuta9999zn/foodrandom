import { create } from 'zustand';

import type { Budget, Cuisine, DishType, Meal, SortKey, SoupType, Style } from '@/lib/types';

export interface FilterState {
  meals: Meal[];
  dishType: DishType | null;
  style: Style | null;
  soupType: SoupType | null;
  cuisines: Cuisine[];
  budgets: Budget[];
  /** `null` = để backend chọn mặc định (gần nhất nếu có vị trí, nổi bật nếu không). */
  sort: SortKey | null;
  page: number;
}

interface FilterActions {
  toggleMeal: (meal: Meal) => void;
  setDishType: (value: DishType | null) => void;
  toggleStyle: (value: Style) => void;
  setSoupType: (value: SoupType | null) => void;
  toggleCuisine: (value: Cuisine) => void;
  clearCuisines: () => void;
  toggleBudget: (value: Budget) => void;
  setSort: (value: SortKey) => void;
  setPage: (page: number) => void;
  reset: () => void;
}

const initial: FilterState = {
  meals: [],
  dishType: null,
  style: null,
  soupType: null,
  cuisines: [],
  budgets: [],
  sort: null,
  page: 1,
};

function toggle<T>(list: T[], value: T): T[] {
  return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

/** Trạng thái bộ lọc tập trung của trang chủ. Đổi bất kỳ tiêu chí nào đều đưa về trang 1. */
export const useFilterStore = create<FilterState & FilterActions>((set) => ({
  ...initial,
  toggleMeal: (meal) => set((s) => ({ meals: toggle(s.meals, meal), page: 1 })),
  setDishType: (dishType) => set({ dishType, page: 1 }),
  toggleStyle: (value) => set((s) => ({ style: s.style === value ? null : value, page: 1 })),
  setSoupType: (soupType) => set({ soupType, page: 1 }),
  toggleCuisine: (value) => set((s) => ({ cuisines: toggle(s.cuisines, value), page: 1 })),
  clearCuisines: () => set({ cuisines: [], page: 1 }),
  toggleBudget: (value) => set((s) => ({ budgets: toggle(s.budgets, value), page: 1 })),
  setSort: (sort) => set({ sort, page: 1 }),
  setPage: (page) => set({ page }),
  reset: () => set(initial),
}));

export function activeFilterCount(s: FilterState) {
  return (
    s.meals.length + s.cuisines.length + s.budgets.length + (s.dishType ? 1 : 0) + (s.style ? 1 : 0) + (s.soupType ? 1 : 0)
  );
}
