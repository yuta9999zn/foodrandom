'use client';

import { keepPreviousData, useMutation, useQuery } from '@tanstack/react-query';
import { Loader2, Sparkles, TriangleAlert } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';

import { FoodCard, FoodCardSkeleton, FoodGrid } from '@/components/food/food-card';
import { EmptyState, ErrorState } from '@/components/states';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { usePrefs } from '@/hooks/use-favorites';
import { useI18n } from '@/i18n/provider';
import { foodsApi, localizedError } from '@/lib/api';
import { CUISINES, PAGE_SIZE, SORTS } from '@/lib/constants';
import type { FoodFilter, RandomResponse, SortKey } from '@/lib/types';
import { cn } from '@/lib/utils';
import { useFilterStore } from '@/stores/filter-store';
import { radiusKm, useLocationStore } from '@/stores/location-store';

import { FilterPanel } from './filter-panel';
import { Pagination } from './pagination';
import { RandomModal, type RandomPhase } from './random-modal';

/** Thời gian tối thiểu của hiệu ứng quay, để kết quả không bật ra quá đột ngột. */
const MIN_SPIN_MS = 1400;
/** Số món vừa chọn được loại trừ ở lần random kế tiếp để tránh lặp. */
const AVOID_REPEAT = 3;
/** Khoảng cách đang được tra ở nền: hỏi lại tối đa ngần này lần, mỗi lần cách 3 giây. */
const PENDING_POLLS = 12;

export function HomeClient() {
  const { t, locale, market } = useI18n();
  const prefs = usePrefs();
  const store = useFilterStore();
  const location = useLocationStore((s) => s.location);
  const clearLocation = useLocationStore((s) => s.clearLocation);
  const radius = useLocationStore((s) => s.radius);
  const resultsRef = useRef<HTMLElement>(null);

  // Đổi quốc gia: mức ngân sách và vị trí cũ không còn đúng nữa.
  const previousMarket = useRef(market);
  useEffect(() => {
    if (previousMarket.current !== market) {
      previousMarket.current = market;
      useFilterStore.setState({ budgets: [], page: 1 });
    }
    if (location?.market && location.market !== market) clearLocation();
  }, [market, location, clearLocation]);

  const filter: FoodFilter = useMemo(
    () => ({
      meals: store.meals,
      dish_type: store.dishType,
      style: store.style,
      soup_type: store.soupType,
      cuisines: store.cuisines,
      budgets: store.budgets,
      lat: location?.lat ?? null,
      lng: location?.lng ?? null,
      radius_km: location ? radiusKm(radius) : null,
    }),
    [store.meals, store.dishType, store.style, store.soupType, store.cuisines, store.budgets, location, radius],
  );

  const polls = useRef(0);
  const list = useQuery({
    queryKey: ['foods', filter, store.sort, store.page, prefs],
    queryFn: ({ signal }) => foodsApi.list({ filter, sort: store.sort, page: store.page }, prefs, signal),
    placeholderData: keepPreviousData,
    // Backend đang tra quán ở nền: hỏi lại sau vài giây để gắn khoảng cách.
    refetchInterval: (query) => {
      if (query.state.data?.location.status !== 'pending') {
        polls.current = 0;
        return false;
      }
      polls.current += 1;
      return polls.current <= PENDING_POLLS ? 3000 : false;
    },
  });

  const data = list.data;
  const items = data?.items ?? [];

  // ----- Random -----
  const [modalOpen, setModalOpen] = useState(false);
  const [phase, setPhase] = useState<RandomPhase>('idle');
  const [result, setResult] = useState<RandomResponse | null>(null);
  const [randomError, setRandomError] = useState<string | null>(null);
  const recentPicks = useRef<string[]>([]);

  const random = useMutation({
    mutationFn: async () => {
      const [response] = await Promise.all([
        foodsApi.random({ filter, sort: data?.sort ?? store.sort, page: store.page, scope: 'page', exclude: recentPicks.current }, prefs),
        new Promise((resolve) => setTimeout(resolve, MIN_SPIN_MS)),
      ]);
      return response;
    },
    onSuccess: (response) => {
      recentPicks.current = [response.food.slug, ...recentPicks.current].slice(0, AVOID_REPEAT);
      setResult(response);
      setPhase('result');
    },
    onError: (error) => {
      setRandomError(localizedError(error, t, locale));
      setPhase('error');
    },
  });

  const startRandom = useCallback(() => {
    if (list.isError) {
      toast.error(t('results.listNotLoaded'));
      return;
    }
    if (data && items.length === 0) {
      toast.error(t('results.noItemsToRandom'));
      return;
    }
    setPhase('spinning');
    setRandomError(null);
    setModalOpen(true);
    random.mutate();
  }, [data, items.length, list.isError, random, t]);

  const changePage = (page: number) => {
    store.setPage(page);
    resultsRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  };

  const sortValue: SortKey = store.sort ?? data?.sort ?? 'featured';
  const sortOptions = SORTS.filter((value) => value !== 'nearest' || !!location);
  const isInitialLoading = list.isPending;
  const isRefreshing = list.isFetching && !list.isPending && data?.location.status !== 'pending';
  const chip = (active: boolean) =>
    cn(
      'shrink-0 rounded-full px-4 py-2 text-[15px] font-semibold transition',
      active ? 'chip-active' : 'bg-surface text-ink ring-1 ring-border hover:bg-primary-50',
    );

  return (
    <>
      <div className="container relative z-10 -mt-24 md:-mt-[11.6vw]">
        <FilterPanel onRandom={startRandom} randomDisabled={isInitialLoading || random.isPending} randomLoading={random.isPending} />
      </div>

      <section ref={resultsRef} aria-labelledby="results-title" className="container mt-10 scroll-mt-24">
        <div className="flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
          <h2 id="results-title" className="flex items-center gap-3 text-[28px] font-bold tracking-tight">
            <Sparkles className="size-8 fill-secondary text-secondary" />
            {t('results.title')}
            {data && <span className="text-base font-medium text-ink-muted">{t('results.count', { n: data.total })}</span>}
          </h2>

          <div className="flex items-center gap-2 rounded-full bg-surface py-1 pl-4 pr-1 shadow-sm ring-1 ring-border">
            <span className="text-sm text-ink-muted">{t('results.sort')}</span>
            <Select value={sortValue} onValueChange={(v) => store.setSort(v as SortKey)}>
              <SelectTrigger
                aria-label={t('results.sortAria')}
                className="h-9 w-auto gap-3 rounded-full border-0 bg-transparent px-3 text-sm font-semibold hover:bg-primary-50"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent align="end">
                {sortOptions.map((value) => (
                  <SelectItem key={value} value={value}>
                    {t(`options.sorts.${value}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Nền ẩm thực: Việt Nam, Nhật Bản, Trung Quốc, Hàn Quốc, Anh */}
        <div role="group" aria-label={t('results.cuisineAria')} className="mt-4 flex gap-2 overflow-x-auto pb-1 scrollbar-none">
          <button type="button" aria-pressed={store.cuisines.length === 0} onClick={store.clearCuisines} className={chip(store.cuisines.length === 0)}>
            {t('filter.all')}
          </button>
          {CUISINES.map((value) => {
            const active = store.cuisines.includes(value);
            return (
              <button key={value} type="button" aria-pressed={active} onClick={() => store.toggleCuisine(value)} className={chip(active)}>
                {t(`options.cuisines.${value}`)}
              </button>
            );
          })}
        </div>

        {data?.location.status === 'pending' && (
          <p role="status" className="mt-4 flex items-center gap-2 rounded-2xl bg-surface px-4 py-3 text-sm font-medium ring-1 ring-border">
            <Loader2 className="size-[18px] shrink-0 animate-spin text-primary" /> {t('results.locationPending')}
          </p>
        )}
        {data?.location.status === 'unavailable' && (
          <p role="status" className="mt-4 flex items-start gap-2 rounded-2xl bg-secondary/20 px-4 py-3 text-sm font-medium">
            <TriangleAlert className="mt-0.5 size-[18px] shrink-0 text-secondary" />
            {t('results.locationUnavailable')}
          </p>
        )}
        {data?.location.status === 'ok' && data.location.is_mock && (
          <p role="status" className="mt-4 flex items-start gap-2 rounded-2xl bg-secondary/20 px-4 py-3 text-sm font-semibold">
            <TriangleAlert className="mt-0.5 size-[18px] shrink-0 text-secondary" />
            {t('results.locationMock')}
          </p>
        )}
        {data && data.relaxed_count > 0 && (
          <p role="status" className="mt-4 text-[15px] text-ink-muted">
            {data.total === 0 ? t('results.relaxedNone') : t('results.relaxedSome', { n: data.total })}{' '}
            {t('results.relaxedTail', { n: data.relaxed_count, size: PAGE_SIZE })}
          </p>
        )}

        <div className="mt-5" aria-busy={list.isFetching}>
          {isInitialLoading ? (
            <FoodGrid>
              {Array.from({ length: PAGE_SIZE }, (_, i) => (
                <FoodCardSkeleton key={i} />
              ))}
            </FoodGrid>
          ) : list.isError ? (
            <ErrorState title={t('states.loadError')} retryLabel={t('states.retry')} message={localizedError(list.error, t, locale)} onRetry={() => list.refetch()} />
          ) : items.length === 0 ? (
            <EmptyState
              title={t('results.emptyTitle')}
              description={t('results.emptyDesc')}
              action={
                <Button variant="outline" onClick={store.reset}>
                  {t('results.clearAll')}
                </Button>
              }
            />
          ) : (
            <FoodGrid className={cn('transition-opacity duration-200', isRefreshing && 'opacity-60')}>
              {items.map((food, index) => (
                <FoodCard key={food.id} food={food} priority={index < 4} />
              ))}
            </FoodGrid>
          )}
        </div>

        {data && <Pagination page={data.page} totalPages={data.total_pages} onChange={changePage} />}
        {data && items.length > 0 && (
          <p className="mt-4 text-center text-sm text-ink-soft">
            {t('results.pageInfo', { page: data.page, pages: data.total_pages, n: items.length })}
          </p>
        )}
      </section>

      <RandomModal
        open={modalOpen}
        onOpenChange={setModalOpen}
        phase={phase}
        candidates={items}
        result={result}
        errorMessage={randomError}
        onAgain={startRandom}
      />
    </>
  );
}
