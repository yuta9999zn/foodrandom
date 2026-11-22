'use client';

import { useQueries } from '@tanstack/react-query';
import { Heart } from 'lucide-react';

import { FoodCard, FoodCardSkeleton, FoodGrid } from '@/components/food/food-card';
import { EmptyState } from '@/components/states';
import { useFavoriteSlugs, usePrefs } from '@/hooks/use-favorites';
import { useI18n } from '@/i18n/provider';
import { ApiError, foodsApi } from '@/lib/api';
import { useFavoritesStore } from '@/stores/favorites-store';

/** Món yêu thích lưu trên trình duyệt này. Thông tin món (tên, giá theo quốc gia) luôn lấy mới từ API. */
export default function CollectionPage() {
  const { t } = useI18n();
  const prefs = usePrefs();
  const slugs = useFavoriteSlugs();
  const remove = useFavoritesStore((s) => s.remove);

  const foods = useQueries({
    queries: slugs.map((slug) => ({
      queryKey: ['food', slug, prefs],
      queryFn: async () => {
        try {
          return await foodsApi.detail(slug, prefs);
        } catch (error) {
          // Món đã bị gỡ khỏi danh mục: bỏ khỏi danh sách đã lưu.
          if (error instanceof ApiError && error.status === 404) remove(slug);
          throw error;
        }
      },
      staleTime: 5 * 60_000,
    })),
  });

  return (
    <div className="container py-10">
      <h1 className="text-3xl font-bold tracking-tight">{t('collection.title')}</h1>
      <p className="mt-1 text-ink-muted">{t('collection.subtitle')}</p>

      <div className="mt-8">
        {slugs.length === 0 ? (
          <EmptyState title={t('collection.foods')} description={t('collection.emptyFoods')} icon={<Heart className="size-8" />} />
        ) : (
          <FoodGrid>
            {foods.map((query, index) =>
              query.data ? (
                <FoodCard key={query.data.slug} food={query.data} />
              ) : query.isPending ? (
                <FoodCardSkeleton key={slugs[index]} />
              ) : null,
            )}
          </FoodGrid>
        )}
      </div>
    </div>
  );
}
