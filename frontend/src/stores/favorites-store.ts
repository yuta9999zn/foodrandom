import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface FavoritesState {
  /** Slug các món đã lưu, mới nhất trước. */
  slugs: string[];
  toggle: (slug: string) => boolean;
  remove: (slug: string) => void;
}

/** Tối đa số món lưu được, để trang Bộ sưu tập không phải tải quá nhiều món một lúc. */
export const MAX_FAVORITES = 60;

/** Món yêu thích lưu ngay trên trình duyệt (localStorage), không cần tài khoản. */
export const useFavoritesStore = create<FavoritesState>()(
  persist(
    (set, get) => ({
      slugs: [],
      toggle: (slug) => {
        const saved = get().slugs.includes(slug);
        set({ slugs: saved ? get().slugs.filter((s) => s !== slug) : [slug, ...get().slugs].slice(0, MAX_FAVORITES) });
        return !saved;
      },
      remove: (slug) => set({ slugs: get().slugs.filter((s) => s !== slug) }),
    }),
    { name: 'foodrandom.favorites', version: 1 },
  ),
);
