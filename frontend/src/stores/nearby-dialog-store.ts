import { create } from 'zustand';

import type { Cuisine } from '@/lib/types';

export interface NearbyTarget {
  slug: string;
  name: string;
  cuisine: Cuisine;
}

interface NearbyDialogState {
  target: NearbyTarget | null;
  open: (target: NearbyTarget) => void;
  close: () => void;
}

/** Hộp thoại "Tìm quán gần nhất" dùng chung cho thẻ món, kết quả random và trang chi tiết. */
export const useNearbyDialog = create<NearbyDialogState>((set) => ({
  target: null,
  open: (target) => set({ target }),
  close: () => set({ target: null }),
}));
