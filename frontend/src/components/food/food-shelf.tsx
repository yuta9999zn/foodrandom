'use client';

import { useQuery } from '@tanstack/react-query';

import { ErrorState } from '@/components/states';
import { usePrefs } from '@/hooks/use-favorites';
import { useI18n } from '@/i18n/provider';
import { foodsApi, localizedError } from '@/lib/api';
import type { Cuisine, SortKey } from '@/lib/types';

import { FoodCard, FoodCardSkeleton, FoodGrid } from './food-card';

/** Một dải món ăn (dùng ở trang Khám phá và Cộng đồng). */
export function FoodShelf({ cuisine, sort, limit = 8, onTotal }: { cuisine?: Cuisine; sort: SortKey; limit?: number; onTotal?: (n: number) => void }) {
  const { t, locale } = useI18n();
  const prefs = usePrefs();
  const query = useQuery({
    queryKey: ['shelf', cuisine, sort, prefs],
    queryFn: async ({ signal }) => {
      const data = await foodsApi.list(
        { filter: { meals: [], dish_type: null, style: null, soup_type: null, cuisines: cuisine ? [cuisine] : [], budgets: [] }, sort, page: 1 },
        prefs,
        signal,
      );
      onTotal?.(data.total);
      return data;
    },
  });

  if (query.isPending)
    return (
      <FoodGrid>
        {Array.from({ length: 4 }, (_, i) => (
          <FoodCardSkeleton key={i} />
        ))}
      </FoodGrid>
    );
  if (query.isError)
    return <ErrorState title={t('states.loadError')} message={localizedError(query.error, t, locale)} retryLabel={t('states.retry')} onRetry={() => query.refetch()} />;
  return (
    <FoodGrid>
      {query.data.items.slice(0, limit).map((food) => (
        <FoodCard key={food.id} food={food} />
      ))}
    </FoodGrid>
  );
}
