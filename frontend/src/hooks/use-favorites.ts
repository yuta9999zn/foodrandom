'use client';

import { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { useI18n } from '@/i18n/provider';
import { useFavoritesStore } from '@/stores/favorites-store';

/** Ngôn ngữ + thị trường hiện tại dưới dạng tham số API. */
export function usePrefs() {
  const { locale, market } = useI18n();
  return { lang: locale, market };
}

/** `true` sau lần render đầu ở trình duyệt — tránh lệch HTML giữa máy chủ và trình duyệt khi đọc localStorage. */
function useHydrated() {
  const [hydrated, setHydrated] = useState(false);
  useEffect(() => setHydrated(true), []);
  return hydrated;
}

/** Slug các món đã lưu trên trình duyệt này (rỗng khi đang render ở máy chủ). */
export function useFavoriteSlugs(): string[] {
  const slugs = useFavoritesStore((s) => s.slugs);
  return useHydrated() ? slugs : [];
}

/** Bật/tắt món yêu thích, kèm thông báo. */
export function useToggleFavoriteFood() {
  const { t } = useI18n();
  const toggle = useFavoritesStore((s) => s.toggle);
  return (slug: string) => {
    const nowSaved = toggle(slug);
    toast.success(nowSaved ? t('favorites.added') : t('favorites.removed'));
  };
}
