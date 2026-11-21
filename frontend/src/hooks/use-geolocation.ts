'use client';

import { useCallback, useState } from 'react';

import { useI18n } from '@/i18n/provider';
import { locationApi } from '@/lib/api';
import { useLocationStore, type UserLocation } from '@/stores/location-store';

export type GeoErrorCode = 'unsupported' | 'denied' | 'unavailable' | 'timeout';

export class GeoError extends Error {
  constructor(public code: GeoErrorCode) {
    super(code);
  }
}

function currentPosition(): Promise<GeolocationPosition> {
  return new Promise((resolve, reject) => {
    if (typeof navigator === 'undefined' || !navigator.geolocation) {
      reject(new GeoError('unsupported'));
      return;
    }
    navigator.geolocation.getCurrentPosition(
      resolve,
      (error) => {
        if (error.code === error.PERMISSION_DENIED) reject(new GeoError('denied'));
        else if (error.code === error.TIMEOUT) reject(new GeoError('timeout'));
        else reject(new GeoError('unavailable'));
      },
      { enableHighAccuracy: true, timeout: 12_000, maximumAge: 5 * 60_000 },
    );
  });
}

/** Lấy vị trí hiện tại bằng Browser Geolocation API, đổi sang tên khu vực rồi lưu vào store. */
export function useGeolocation() {
  const setLocation = useLocationStore((s) => s.setLocation);
  const { t, locale } = useI18n();
  const [locating, setLocating] = useState(false);

  const locate = useCallback(async (): Promise<UserLocation> => {
    setLocating(true);
    try {
      const position = await currentPosition();
      const { latitude: lat, longitude: lng } = position.coords;
      let label = t('geo.currentLocation');
      try {
        const reverse = await locationApi.reverse(lat, lng, locale);
        if (reverse.label) label = reverse.label;
      } catch {
        // Không đổi được tên khu vực thì vẫn dùng toạ độ — không ảnh hưởng việc tìm quán.
      }
      const location: UserLocation = { lat, lng, label, source: 'gps' };
      setLocation(location);
      return location;
    } finally {
      setLocating(false);
    }
  }, [setLocation, t, locale]);

  /** Thông báo lỗi định vị theo ngôn ngữ hiện tại. */
  const geoErrorMessage = useCallback((error: unknown) => t(`geo.${error instanceof GeoError ? error.code : 'unavailable'}`), [t]);

  return { locate, locating, geoErrorMessage };
}
