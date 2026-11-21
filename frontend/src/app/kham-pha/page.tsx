'use client';

import { ArrowRight } from 'lucide-react';
import Link from 'next/link';

import { FoodShelf } from '@/components/food/food-shelf';
import { Flag } from '@/components/flag';
import { useI18n } from '@/i18n/provider';
import { CUISINES } from '@/lib/constants';
import { useFilterStore } from '@/stores/filter-store';

const FLAGS = { vietnam: 'VN', japan: 'JP', china: 'CN', korea: 'KR', uk: 'GB' } as const;

export default function ExplorePage() {
  const { t } = useI18n();

  return (
    <div className="container py-10">
      <h1 className="text-3xl font-bold tracking-tight sm:text-4xl">{t('explore.title')}</h1>
      <p className="mt-2 max-w-2xl text-[17px] text-ink-muted">{t('explore.subtitle')}</p>

      <nav className="mt-6 flex flex-wrap gap-2" aria-label={t('results.cuisineAria')}>
        {CUISINES.map((c) => (
          <a key={c} href={`#${c}`} className="rounded-full bg-surface px-4 py-2 font-semibold ring-1 ring-border transition hover:bg-primary-50">
            <span className="inline-flex items-center gap-2">
              <Flag code={FLAGS[c]} /> {t(`options.cuisines.${c}`)}
            </span>
          </a>
        ))}
      </nav>

      {CUISINES.map((cuisine) => (
        <section key={cuisine} id={cuisine} aria-labelledby={`${cuisine}-title`} className="mt-12 scroll-mt-24">
          <div className="mb-5 flex items-end justify-between gap-4">
            <h2 id={`${cuisine}-title`} className="text-2xl font-bold">
              <span className="inline-flex items-center gap-3">
                <Flag code={FLAGS[cuisine]} /> {t(`options.cuisines.${cuisine}`)}
              </span>
            </h2>
            <Link
              href="/"
              onClick={() => useFilterStore.setState({ cuisines: [cuisine], page: 1 })}
              className="flex items-center gap-1 text-[15px] font-semibold text-primary-700 hover:underline"
            >
              {t('explore.viewAll')} <ArrowRight className="size-4" />
            </Link>
          </div>
          <FoodShelf cuisine={cuisine} sort="featured" limit={8} />
        </section>
      ))}
    </div>
  );
}
