'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { usePathname, useRouter } from 'next/navigation';
import { toast } from 'sonner';

import { useI18n } from '@/i18n/provider';
import { favoritesApi, localizedError } from '@/lib/api';

import { useMe } from './use-auth';

const SLUGS_KEY = ['favorites', 'food-slugs'] as const;

/** Ngôn ngữ + thị trường hiện tại dưới dạng tham số API. */
export function usePrefs() {
  const { locale, market } = useI18n();
  return { lang: locale, market };
}

/** Slug các món đã lưu của người dùng hiện tại (rỗng nếu là khách). */
export function useFavoriteSlugs() {
  const { data: me } = useMe();
  return useQuery({
    queryKey: SLUGS_KEY,
    queryFn: async () => (await favoritesApi.foodSlugs()).slugs,
    enabled: !!me,
    staleTime: 60_000,
  });
}

/** Đưa khách tới trang đăng nhập rồi quay lại đúng trang đang xem. */
export function useRequireLogin() {
  const { data: me } = useMe();
  const { t } = useI18n();
  const router = useRouter();
  const pathname = usePathname();
  return (actionKey: 'actionSaveFood' | 'actionSavePlace' | 'actionRate') => {
    if (me) return true;
    toast.info(t('favorites.loginTo', { action: t(`favorites.${actionKey}`) }), {
      action: { label: t('nav.login'), onClick: () => router.push(`/dang-nhap?next=${encodeURIComponent(pathname)}`) },
    });
    return false;
  };
}

/** Bật/tắt món yêu thích với cập nhật lạc quan; lỗi thì hoàn tác. */
export function useToggleFavoriteFood() {
  const queryClient = useQueryClient();
  const requireLogin = useRequireLogin();
  const { t, locale } = useI18n();

  const mutation = useMutation({
    mutationFn: ({ slug, favorited }: { slug: string; favorited: boolean }) =>
      favorited ? favoritesApi.removeFood(slug) : favoritesApi.addFood(slug),
    onMutate: async ({ slug, favorited }) => {
      await queryClient.cancelQueries({ queryKey: SLUGS_KEY });
      const previous = queryClient.getQueryData<string[]>(SLUGS_KEY) ?? [];
      queryClient.setQueryData<string[]>(SLUGS_KEY, favorited ? previous.filter((s) => s !== slug) : [slug, ...previous]);
      return { previous };
    },
    onError: (error, _vars, context) => {
      if (context) queryClient.setQueryData(SLUGS_KEY, context.previous);
      toast.error(localizedError(error, t, locale));
    },
    onSuccess: (_data, { favorited }) => {
      toast.success(favorited ? t('favorites.removed') : t('favorites.added'));
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['favorites'] });
    },
  });

  return (slug: string, favorited: boolean) => {
    if (!requireLogin('actionSaveFood')) return;
    mutation.mutate({ slug, favorited });
  };
}
