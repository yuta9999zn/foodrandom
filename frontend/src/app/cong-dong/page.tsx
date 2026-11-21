'use client';

import { Heart, MessageCircle, Star } from 'lucide-react';

import { FoodShelf } from '@/components/food/food-shelf';
import { useI18n } from '@/i18n/provider';

export default function CommunityPage() {
  const { t } = useI18n();
  return (
    <div className="container py-10">
      <h1 className="text-3xl font-bold tracking-tight sm:text-4xl">{t('community.title')}</h1>
      <p className="mt-2 text-[17px] text-ink-muted">{t('community.subtitle')}</p>

      <section className="mt-10" aria-labelledby="popular-title">
        <h2 id="popular-title" className="mb-5 flex items-center gap-2 text-2xl font-bold">
          <Heart className="size-6 fill-primary text-primary" /> {t('community.mostFavorited')}
        </h2>
        <FoodShelf sort="popular" limit={8} />
      </section>

      <section className="mt-12" aria-labelledby="rated-title">
        <h2 id="rated-title" className="mb-5 flex items-center gap-2 text-2xl font-bold">
          <Star className="size-6 fill-secondary text-secondary" /> {t('community.topRated')}
        </h2>
        <FoodShelf sort="rating" limit={8} />
      </section>

      <p className="mt-12 flex items-center justify-center gap-2 rounded-3xl border border-dashed border-border bg-surface/70 px-6 py-8 text-center text-ink-muted">
        <MessageCircle className="size-5 shrink-0 text-primary" /> {t('community.soon')}
      </p>
    </div>
  );
}
