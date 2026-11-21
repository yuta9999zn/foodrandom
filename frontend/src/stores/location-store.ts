import { create } from 'zustand';
import { persist } from 'zustand/middleware';

import type { MarketCode } from '@/i18n/config';

/** Bán kính tìm quán (km); `null` = không giới hạn (tối đa theo nhà cung cấp địa điểm). */
export type Radius = number | null;

/** Các mức chọn nhanh và giới hạn của thanh kéo. Google Places nhận tối đa 50 km. */
export const RADIUS_PRESETS = [1, 2, 3, 5, 10, 20] as const;
export const MIN_RADIUS_KM = 1;
export const MAX_RADIUS_KM = 50;
export const DEFAULT_RADIUS_KM = 5;

export interface UserLocation {
  lat: number;
  lng: number;
  /** Tên khu vực hiển thị trong ô địa điểm. */
  label: string;
  /** `gps`: Browser Geolocation API. `manual`: người dùng nhập địa chỉ. */
  source: 'gps' | 'manual';
  /** Thị trường lúc chọn vị trí; đổi quốc gia thì vị trí cũ bị bỏ. */
  market?: MarketCode;
}

interface LocationState {
  location: UserLocation | null;
  radius: Radius;
  setLocation: (location: UserLocation) => void;
  clearLocation: () => void;
  setRadius: (radius: Radius) => void;
}

export function clampRadius(km: number) {
  return Math.min(MAX_RADIUS_KM, Math.max(MIN_RADIUS_KM, Math.round(km)));
}

/** Vị trí người dùng, lưu ở localStorage để lần sau không phải hỏi lại. Chỉ gửi lên backend khi tìm món/quán. */
export const useLocationStore = create<LocationState>()(
  persist(
    (set) => ({
      location: null,
      radius: DEFAULT_RADIUS_KM,
      setLocation: (location) => set({ location }),
      clearLocation: () => set({ location: null }),
      setRadius: (radius) => set({ radius: radius == null ? null : clampRadius(radius) }),
    }),
    {
      name: 'foodrandom.location',
      version: 3,
      // Bản cũ lưu bán kính dạng chuỗi ('1' | '3' | … | 'none').
      migrate: (persisted, version) => {
        const state = (persisted ?? {}) as { location?: UserLocation | null; radius?: unknown };
        if (version < 2) return { location: null, radius: DEFAULT_RADIUS_KM } as never;
        const old = state.radius;
        const radius = old === 'none' || old === null ? null : Number.isFinite(Number(old)) ? clampRadius(Number(old)) : DEFAULT_RADIUS_KM;
        return { location: state.location ?? null, radius } as never;
      },
    },
  ),
);

export const radiusKm = (radius: Radius) => radius;
