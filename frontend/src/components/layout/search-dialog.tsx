'use client';

import { useQuery } from '@tanstack/react-query';
import { Flame, Loader2, Search } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';
import { useEffect, useState } from 'react';

import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { PriceText } from '@/components/food/food-card';
import { usePrefs } from '@/hooks/use-favorites';
import { useI18n } from '@/i18n/provider';
import { foodsApi, localizedError } from '@/lib/api';

function useDebounced<T>(value: T, delayMs: number) {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delayMs);
    return () => clearTimeout(timer);
  }, [value, delayMs]);
  return debounced;
}

/** Tìm nhanh món ăn theo tên (không cần gõ dấu). */
export function SearchDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const { t, locale } = useI18n();
  const prefs = usePrefs();
  const [text, setText] = useState('');
  const q = useDebounced(text.trim(), 250);

  const { data, isFetching, error } = useQuery({
    queryKey: ['search', q, prefs],
    queryFn: ({ signal }) =>
      foodsApi.list(
        { filter: { meals: [], dish_type: null, style: null, soup_type: null, cuisines: [], budgets: [], q }, sort: null, page: 1 },
        prefs,
        signal,
      ),
    enabled: open && q.length >= 2,
  });

  const results = data?.items.filter((item) => item.match_level === 'exact') ?? [];

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        onOpenChange(next);
        if (!next) setText('');
      }}
    >
      <DialogContent className="top-[12dvh] max-w-xl translate-y-0 p-0">
        <DialogTitle className="sr-only">{t('search.title')}</DialogTitle>
        <DialogDescription className="sr-only">{t('search.desc')}</DialogDescription>
        <div className="flex items-center gap-3 border-b border-border px-5 py-4 pr-16">
          {isFetching ? <Loader2 className="size-5 animate-spin text-primary" /> : <Search className="size-5 text-ink-muted" />}
          <input
            autoFocus
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={t('search.placeholder')}
            className="h-9 w-full bg-transparent text-[17px] outline-none placeholder:text-ink-soft focus-visible:ring-0"
            aria-label={t('search.aria')}
          />
        </div>

        <div className="max-h-[56dvh] overflow-y-auto p-2">
          {q.length < 2 ? (
            <p className="px-4 py-8 text-center text-[15px] text-ink-muted">{t('search.minChars')}</p>
          ) : error ? (
            <p role="alert" className="px-4 py-8 text-center text-[15px] text-primary-700">
              {localizedError(error, t, locale)}
            </p>
          ) : results.length === 0 && !isFetching ? (
            <p className="px-4 py-8 text-center text-[15px] text-ink-muted">{t('search.noResult', { q })}</p>
          ) : (
            <ul>
              {results.map((food) => (
                <li key={food.id}>
                  <Link
                    href={`/foods/${food.slug}`}
                    onClick={() => onOpenChange(false)}
                    className="flex items-center gap-3 rounded-2xl p-2.5 transition hover:bg-primary-50"
                  >
                    <Image src={food.image_url} alt="" width={64} height={64} className="size-16 rounded-xl object-cover" />
                    <span className="min-w-0 flex-1">
                      <span className="block truncate font-semibold">{food.name}</span>
                      <span className="flex flex-wrap items-center gap-x-1.5 text-sm text-ink-muted">
                        {t(`options.cuisines.${food.cuisine}`)} · <PriceText food={food} className="inline text-sm" />
                      </span>
                    </span>
                    <span className="flex items-center gap-1 text-sm font-medium text-ink-muted">
                      <Flame className="size-4 text-secondary" /> {food.calories_kcal} kcal
                    </span>
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
