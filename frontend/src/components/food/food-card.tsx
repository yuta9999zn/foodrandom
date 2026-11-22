'use client';

import { FlaskConical, Flame, Footprints, Heart, Leaf, Soup, Sparkles, Star, Utensils } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';

import { useFavoriteSlugs, useToggleFavoriteFood } from '@/hooks/use-favorites';
import { useI18n } from '@/i18n/provider';
import { isSoup } from '@/lib/constants';
import type { FoodListItem, FoodSummary } from '@/lib/types';
import { cn, formatCount, formatDistance } from '@/lib/utils';

import { MapsSearchIcon } from './maps-search-button';

export function StyleChip({ style }: { style: FoodSummary['style'] }) {
  const { t } = useI18n();
  const traditional = style === 'traditional';
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[13px] font-medium',
        traditional ? 'bg-leaf-soft text-leaf' : 'bg-primary-50 text-primary-700',
      )}
    >
      {traditional ? <Leaf className="size-3.5" /> : <FlaskConical className="size-3.5" />}
      {t(`options.styles.${style}`)}
    </span>
  );
}

export function TypeChip({ food }: { food: Pick<FoodSummary, 'soup_type' | 'dish_type'> }) {
  const { t } = useI18n();
  const soup = isSoup(food.soup_type);
  return (
    <span className="inline-flex items-center gap-1.5 rounded-full bg-primary-50 px-2.5 py-1 text-[13px] font-medium text-primary-700">
      {soup ? <Soup className="size-3.5" /> : <Utensils className="size-3.5" />}
      {soup ? t('card.soup') : t(`options.dishTypes.${food.dish_type}`)}
    </span>
  );
}

export function FavoriteButton({ slug, name, className }: { slug: string; name: string; className?: string }) {
  const { t } = useI18n();
  const slugs = useFavoriteSlugs();
  const toggle = useToggleFavoriteFood();
  const favorited = slugs.includes(slug);
  return (
    <button
      type="button"
      onClick={(event) => {
        event.preventDefault();
        toggle(slug);
      }}
      aria-pressed={favorited}
      aria-label={favorited ? t('card.favRemove', { name }) : t('card.favAdd', { name })}
      className={cn(
        'grid size-10 place-items-center rounded-full text-white transition hover:scale-110 [&_svg]:drop-shadow-[0_1px_3px_rgba(0,0,0,0.6)]',
        className,
      )}
    >
      <Heart className={cn('size-[26px]', favorited && 'fill-primary text-primary')} strokeWidth={2.2} />
    </button>
  );
}

export function RatingLine({ food, className }: { food: Pick<FoodSummary, 'rating_avg' | 'rating_count'>; className?: string }) {
  const { formatNumber } = useI18n();
  if (food.rating_avg == null || food.rating_count === 0) return null;
  return (
    <span className={cn('inline-flex items-center gap-1.5 font-semibold', className)}>
      <Star className="size-[18px] fill-secondary text-secondary" />
      {food.rating_avg.toFixed(1)}
      <span className="font-normal text-ink-muted">({formatCount(food.rating_count, formatNumber)})</span>
    </span>
  );
}

/** Giá theo tiền tệ của thị trường; món không bán ở thị trường này hiện ghi chú thay cho giá. */
export function PriceText({ food, className }: { food: Pick<FoodSummary, 'price_min' | 'price_max'>; className?: string }) {
  const { formatPrice, t, market } = useI18n();
  const price = formatPrice(food.price_min, food.price_max);
  if (!price) {
    return (
      <span className={cn('block text-[15px] font-medium text-ink-muted', className)}>
        {t('card.notSold', { country: t(`market.names.${market}`) })}
      </span>
    );
  }
  return <span className={cn('block font-bold text-primary-600', className)}>{price}</span>;
}

/** Thẻ món ăn trong lưới kết quả. Cả thẻ dẫn tới trang chi tiết; các nút phụ nằm phía trên lớp liên kết. */
export function FoodCard({ food, priority }: { food: FoodListItem | FoodSummary; priority?: boolean }) {
  const { t } = useI18n();
  const place = 'nearest_place' in food ? food.nearest_place : null;
  const relaxed = 'match_level' in food && food.match_level === 'relaxed';

  return (
    <article className="group relative flex flex-col overflow-hidden rounded-2xl bg-surface shadow-card ring-1 ring-border/70 transition duration-300 hover:-translate-y-1 hover:shadow-[0_18px_40px_-14px_rgba(120,60,70,0.4)]">
      <div className="relative aspect-[2/1] overflow-hidden bg-primary-50">
        <Image
          src={food.image_url}
          alt={food.name}
          fill
          priority={priority}
          sizes="(min-width: 1280px) 340px, (min-width: 1024px) 31vw, (min-width: 640px) 46vw, 94vw"
          className="object-cover transition duration-500 group-hover:scale-105"
        />
        <div className="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/45 via-transparent to-black/20" />

        {food.featured ? (
          <span className="absolute left-3 top-3 inline-flex items-center gap-1.5 rounded-full bg-primary px-3 py-1.5 text-[13px] font-semibold text-white shadow-md">
            <Sparkles className="size-3.5 fill-secondary text-secondary" /> {t('card.featured')}
          </span>
        ) : relaxed ? (
          <span className="absolute left-3 top-3 rounded-full bg-dark/75 px-3 py-1.5 text-[13px] font-medium text-white backdrop-blur">
            {t('card.relaxed')}
          </span>
        ) : null}

        <FavoriteButton slug={food.slug} name={food.name} className="absolute right-2 top-2 z-10" />

        <span className="absolute bottom-3 right-3 rounded-full bg-white/90 px-2.5 py-1 text-[12px] font-semibold text-ink shadow-sm backdrop-blur">
          {t(`options.cuisines.${food.cuisine}`)}
        </span>

        {place && (
          <span className="absolute bottom-3 left-3 inline-flex items-center gap-1.5 rounded-full bg-dark/75 px-2.5 py-1 text-[13px] font-semibold text-white backdrop-blur">
            <Footprints className="size-3.5" /> {formatDistance(place.distance_km)}
          </span>
        )}
      </div>

      <div className="flex flex-1 flex-col px-5 pb-4 pt-3.5">
        <h3 className="line-clamp-1 text-[19px] font-bold leading-snug" title={food.name}>
          <Link href={`/foods/${food.slug}`} className="after:absolute after:inset-0 after:content-[''] focus-visible:ring-0">
            {food.name}
          </Link>
        </h3>

        <div className="mt-2 flex flex-wrap gap-1.5">
          <StyleChip style={food.style} />
          <TypeChip food={food} />
        </div>

        <div className="mt-3 flex items-end justify-between gap-3">
          <div className="min-w-0">
            <PriceText food={food} className="text-[19px]" />
            <p className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-[15px]">
              <RatingLine food={food} />
              <span className="inline-flex items-center gap-1 font-medium text-ink-muted">
                <Flame className="size-4 text-secondary" /> {food.calories_kcal} kcal
              </span>
            </p>
          </div>
          <MapsSearchIcon name={food.name} />
        </div>

        {place && (
          <p className="mt-2.5 line-clamp-1 border-t border-border/70 pt-2.5 text-[13px] text-ink-muted" title={place.address ?? place.name}>
            <span className="font-semibold text-ink">{place.name}</span>
            {place.address && ` · ${place.address}`}
            {place.is_mock && ` · ${t('card.mock')}`}
          </p>
        )}
      </div>
    </article>
  );
}

export function FoodCardSkeleton() {
  return (
    <div className="overflow-hidden rounded-2xl bg-surface shadow-card ring-1 ring-border/70" aria-hidden="true">
      <div className="skeleton aspect-[2/1] rounded-none" />
      <div className="space-y-3 px-5 pb-5 pt-4">
        <div className="skeleton h-6 w-3/4" />
        <div className="flex gap-2">
          <div className="skeleton h-6 w-24 rounded-full" />
          <div className="skeleton h-6 w-20 rounded-full" />
        </div>
        <div className="skeleton h-6 w-1/2" />
        <div className="skeleton h-5 w-1/3" />
      </div>
    </div>
  );
}

export function FoodGrid({ children, className }: { children: React.ReactNode; className?: string }) {
  return <div className={cn('grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4', className)}>{children}</div>;
}
