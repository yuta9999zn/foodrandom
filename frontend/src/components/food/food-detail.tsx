'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { ChefHat, Clock, Flame, Globe2, Info, Loader2, MapPinned, Navigation, Share2, Shuffle, Star, UsersRound } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { toast } from 'sonner';

import { LocationPicker } from '@/components/location/location-picker';
import { NearbyPlaces } from '@/components/location/nearby-places';
import { Button } from '@/components/ui/button';
import { useMe } from '@/hooks/use-auth';
import { usePrefs, useRequireLogin } from '@/hooks/use-favorites';
import { useI18n } from '@/i18n/provider';
import { foodsApi, localizedError } from '@/lib/api';
import type { FoodDetail } from '@/lib/types';
import { cn, formatCount, formatMinutes } from '@/lib/utils';
import { useLocationStore } from '@/stores/location-store';

import { FavoriteButton, FoodCard, FoodGrid, StyleChip, TypeChip } from './food-card';

function Stat({ icon: Icon, label, value }: { icon: typeof Clock; label: string; value: string }) {
  return (
    <div className="flex items-center gap-3 rounded-2xl bg-surface p-3.5 ring-1 ring-border">
      <div className="grid size-11 shrink-0 place-items-center rounded-full bg-primary-50 text-primary">
        <Icon className="size-5" />
      </div>
      <div className="min-w-0">
        <p className="text-sm text-ink-muted">{label}</p>
        <p className="truncate font-semibold" title={value}>
          {value}
        </p>
      </div>
    </div>
  );
}

/** Calo và tỉ lệ năng lượng từ đạm / tinh bột / chất béo. */
function NutritionCard({ food }: { food: FoodDetail }) {
  const { t } = useI18n();
  const { protein_g, carbs_g, fat_g, calories_kcal, serving_note } = food.nutrition;
  const parts = [
    { label: t('detail.protein'), grams: protein_g, kcal: protein_g * 4, color: 'bg-primary' },
    { label: t('detail.carbs'), grams: carbs_g, kcal: carbs_g * 4, color: 'bg-secondary' },
    { label: t('detail.fat'), grams: fat_g, kcal: fat_g * 9, color: 'bg-leaf' },
  ];
  const total = parts.reduce((sum, p) => sum + p.kcal, 0) || 1;

  return (
    <section aria-labelledby="nutrition-title" className="rounded-3xl bg-surface p-6 shadow-card ring-1 ring-border">
      <h2 id="nutrition-title" className="flex items-center gap-2 text-xl font-bold">
        <Flame className="size-6 text-secondary" /> {t('detail.nutrition')}
      </h2>
      <p className="mt-4 flex items-baseline gap-2">
        <span className="text-5xl font-bold tracking-tight text-primary-600">{calories_kcal}</span>
        <span className="text-lg font-semibold">kcal</span>
      </p>
      <p className="mt-1 text-sm text-ink-muted">
        {t('detail.perServing')}
        {serving_note ? ` (${serving_note})` : ''}
      </p>

      <div className="mt-5 flex h-3 overflow-hidden rounded-full bg-background" aria-hidden="true">
        {parts.map((p) => (
          <div key={p.label} className={p.color} style={{ width: `${(p.kcal / total) * 100}%` }} />
        ))}
      </div>
      <dl className="mt-4 grid grid-cols-3 gap-2 text-center">
        {parts.map((p) => (
          <div key={p.label} className="rounded-2xl bg-background px-2 py-3">
            <dt className="flex items-center justify-center gap-1.5 text-[13px] text-ink-muted">
              <span className={cn('size-2.5 shrink-0 rounded-full', p.color)} /> {p.label}
            </dt>
            <dd className="mt-1 text-lg font-bold">{p.grams} g</dd>
            <dd className="text-xs text-ink-soft">{t('detail.energyPct', { n: Math.round((p.kcal / total) * 100) })}</dd>
          </div>
        ))}
      </dl>
      <p className="mt-4 flex gap-2 text-xs leading-relaxed text-ink-soft">
        <Info className="mt-0.5 size-3.5 shrink-0" />
        {t('detail.nutritionNote')}
      </p>
    </section>
  );
}

function RatingCard({ food }: { food: FoodDetail }) {
  const { t, locale, formatNumber } = useI18n();
  const { data: me } = useMe();
  const requireLogin = useRequireLogin();
  const queryClient = useQueryClient();
  const [hover, setHover] = useState(0);
  const key = ['rating', food.slug, me?.id ?? 'guest'];

  const rating = useQuery({
    queryKey: key,
    queryFn: () => foodsApi.rating(food.slug),
    initialData: me ? undefined : { rating_avg: food.rating_avg, rating_count: food.rating_count, mine: null },
  });
  const rate = useMutation({
    mutationFn: (score: number) => foodsApi.rate(food.slug, score),
    onSuccess: (data) => {
      queryClient.setQueryData(key, data);
      toast.success(t('detail.thanks'));
    },
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });

  const mine = rating.data?.mine ?? 0;
  const shown = hover || mine;

  return (
    <section aria-labelledby="rating-title" className="rounded-3xl bg-surface p-6 shadow-card ring-1 ring-border">
      <h2 id="rating-title" className="flex items-center gap-2 text-xl font-bold">
        <Star className="size-6 fill-secondary text-secondary" /> {t('detail.rating')}
      </h2>
      {rating.data?.rating_avg != null && rating.data.rating_count > 0 ? (
        <p className="mt-3">
          <span className="text-3xl font-bold">{rating.data.rating_avg.toFixed(1)}</span>
          <span className="text-ink-muted"> {t('detail.ratingSummary', { n: formatCount(rating.data.rating_count, formatNumber) })}</span>
        </p>
      ) : (
        <p className="mt-3 text-[15px] text-ink-muted">{t('detail.noRating')}</p>
      )}
      <div className="mt-4 flex items-center gap-1" role="radiogroup" aria-label={t('detail.rateAria')} onMouseLeave={() => setHover(0)}>
        {[1, 2, 3, 4, 5].map((score) => (
          <button
            key={score}
            type="button"
            role="radio"
            aria-checked={mine === score}
            aria-label={t('detail.stars', { n: score })}
            disabled={rate.isPending}
            onMouseEnter={() => setHover(score)}
            onClick={() => requireLogin('actionRate') && rate.mutate(score)}
            className="rounded-full p-1 transition hover:scale-110"
          >
            <Star className={cn('size-8', score <= shown ? 'fill-secondary text-secondary' : 'text-border')} />
          </button>
        ))}
        {rate.isPending && <Loader2 className="ml-2 size-5 animate-spin text-primary" />}
      </div>
      <p className="mt-1 text-sm text-ink-muted">{mine ? t('detail.youRated', { n: mine }) : t('detail.tapToRate')}</p>
    </section>
  );
}

function PlacesSection({ food }: { food: FoodDetail }) {
  const { t } = useI18n();
  const location = useLocationStore((s) => s.location);
  const [asking, setAsking] = useState(false);
  const target = { slug: food.slug, name: food.name, cuisine: food.cuisine };

  return (
    <section id="dia-diem" aria-labelledby="places-title" className="scroll-mt-24 rounded-3xl bg-surface p-6 shadow-card ring-1 ring-border">
      <h2 id="places-title" className="flex items-center gap-2 text-xl font-bold">
        <MapPinned className="size-6 text-primary" /> {t('detail.placesTitle')}
      </h2>
      {location ? (
        <>
          <p className="mb-4 mt-1 text-sm text-ink-muted">
            {t('detail.around')} <span className="font-semibold text-ink">{location.label}</span>
          </p>
          <NearbyPlaces target={target} limit={8} />
        </>
      ) : asking ? (
        <div className="mt-4">
          <LocationPicker onPicked={() => setAsking(false)} />
        </div>
      ) : (
        <div className="mt-3">
          <p className="text-[15px] text-ink-muted">{t('detail.placesAsk', { name: food.name })}</p>
          <Button className="mt-4" onClick={() => setAsking(true)}>
            <Navigation /> {t('detail.findNearMe')}
          </Button>
        </div>
      )}
    </section>
  );
}

export function FoodDetailView({ food }: { food: FoodDetail }) {
  const { t, locale, market, formatPrice } = useI18n();
  const prefs = usePrefs();
  const router = useRouter();
  const [randomizing, setRandomizing] = useState(false);
  const totalMinutes = food.prep_minutes + food.cook_minutes;
  const price = formatPrice(food.price_min, food.price_max);
  const country = t(`market.names.${market}`);

  const share = async () => {
    const url = window.location.href;
    const data = { title: `${food.name} · FoodRandom`, text: food.description, url };
    try {
      if (navigator.share) {
        await navigator.share(data);
        return;
      }
      await navigator.clipboard.writeText(url);
      toast.success(t('detail.copied'));
    } catch (error) {
      if ((error as DOMException)?.name !== 'AbortError') toast.error(t('detail.shareFailed'));
    }
  };

  /** Random món tương tự: cùng nền ẩm thực và loại món, trừ món đang xem. */
  const randomSimilar = async () => {
    setRandomizing(true);
    try {
      const result = await foodsApi.random(
        { filter: { cuisines: [food.cuisine], dish_type: food.dish_type }, scope: 'all', exclude: [food.slug] },
        prefs,
      );
      const next = result.food.slug === food.slug ? food.similar[0]?.slug : result.food.slug;
      if (next) router.push(`/foods/${next}`);
      else toast.info(t('detail.noSimilar'));
    } catch (error) {
      toast.error(localizedError(error, t, locale));
    } finally {
      setRandomizing(false);
    }
  };

  return (
    <article>
      <header className="relative isolate overflow-hidden bg-dark text-white">
        <Image src={food.image_url} alt="" fill priority sizes="100vw" className="-z-10 object-cover opacity-35 blur-sm" />
        <div className="absolute inset-0 -z-10 bg-gradient-to-t from-dark via-dark/70 to-dark/40" />
        <div className="container grid gap-8 py-10 lg:grid-cols-[1.05fr_1fr] lg:items-center lg:py-14">
          <div className="relative aspect-[4/3] overflow-hidden rounded-3xl shadow-glass ring-1 ring-white/15">
            <Image src={food.image_url} alt={food.name} fill priority sizes="(min-width: 1024px) 50vw, 100vw" className="object-cover" />
            <FavoriteButton slug={food.slug} name={food.name} className="absolute right-3 top-3 size-12 bg-black/25 backdrop-blur" />
          </div>

          <div>
            <nav aria-label="Breadcrumb" className="text-sm text-white/70">
              <Link href="/" className="hover:text-white">
                {t('nav.home')}
              </Link>{' '}
              /{' '}
              <Link href={`/kham-pha#${food.cuisine}`} className="hover:text-white">
                {t('detail.breadcrumbCuisine', { cuisine: t(`options.cuisines.${food.cuisine}`) })}
              </Link>
            </nav>
            <h1 className="mt-3 text-4xl font-bold leading-tight tracking-tight sm:text-5xl">{food.name}</h1>
            {food.original_name && food.original_name !== food.name && <p className="mt-2 text-xl text-white/80">{food.original_name}</p>}

            <div className="mt-4 flex flex-wrap gap-2">
              <StyleChip style={food.style} />
              <TypeChip food={food} />
              {food.soup_type !== 'kho' && (
                <span className="inline-flex items-center rounded-full bg-white/15 px-2.5 py-1 text-[13px] font-medium">
                  {t(`options.soups.${food.soup_type}`)}
                </span>
              )}
              {food.meals.map((meal) => (
                <span key={meal} className="inline-flex items-center rounded-full bg-white/15 px-2.5 py-1 text-[13px] font-medium">
                  {t('detail.meal', { meal: t(`options.meals.${meal}`) })}
                </span>
              ))}
            </div>

            <p className="mt-5 text-[17px] leading-relaxed text-white/90">{food.description}</p>

            <div className="mt-5 flex flex-wrap items-center gap-x-6 gap-y-2">
              {price ? (
                <span className="flex flex-col">
                  <span className="text-xs uppercase tracking-wide text-white/60">{t('detail.priceIn', { country })}</span>
                  <span className="text-3xl font-bold text-primary-300">{price}</span>
                </span>
              ) : (
                <span className="max-w-sm text-[15px] text-white/75">{t('detail.notSoldLong', { country })}</span>
              )}
              <span className="flex items-center gap-1.5 text-lg font-semibold">
                <Flame className="size-5 text-secondary" /> {food.calories_kcal} kcal
              </span>
            </div>

            <div className="mt-6 flex flex-wrap gap-2.5">
              <Button size="lg" asChild>
                <a href="#dia-diem">
                  <Navigation /> {t('detail.findPlaces')}
                </a>
              </Button>
              <Button
                size="lg"
                variant="outline"
                className="border-white/30 bg-white/10 text-white hover:bg-white/20"
                onClick={randomSimilar}
                disabled={randomizing}
              >
                {randomizing ? <Loader2 className="animate-spin" /> : <Shuffle />} {t('detail.randomSimilar')}
              </Button>
              <Button size="lg" variant="ghost" className="text-white hover:bg-white/15" onClick={share}>
                <Share2 /> {t('detail.share')}
              </Button>
            </div>
          </div>
        </div>
      </header>

      <div className="container mt-10 grid gap-8 lg:grid-cols-[1.6fr_1fr]">
        <div className="space-y-8">
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            <Stat icon={Globe2} label={t('detail.cuisine')} value={t(`options.cuisines.${food.cuisine}`)} />
            <Stat icon={ChefHat} label={t('detail.cooking')} value={food.cooking_style || '—'} />
            <Stat icon={Clock} label={t('detail.time')} value={totalMinutes ? formatMinutes(totalMinutes, t) : '—'} />
            <Stat icon={UsersRound} label={t('detail.servings')} value={t('detail.persons', { n: food.servings })} />
          </div>

          {food.origin && (
            <section aria-labelledby="origin-title">
              <h2 id="origin-title" className="text-2xl font-bold">
                {t('detail.origin')}
              </h2>
              <p className="mt-3 text-[17px] leading-relaxed text-ink/90">{food.origin}</p>
            </section>
          )}

          <section aria-labelledby="ingredients-title" className="rounded-3xl bg-surface p-6 shadow-card ring-1 ring-border">
            <h2 id="ingredients-title" className="text-2xl font-bold">
              {t('detail.ingredients')}
            </h2>
            <p className="mt-1 text-sm text-ink-muted">
              {t('detail.forPeople', { n: food.servings })} · {t('detail.prep', { t: formatMinutes(food.prep_minutes, t) })}
              {food.cook_minutes > 0 && ` · ${t('detail.cook', { t: formatMinutes(food.cook_minutes, t) })}`}
            </p>
            <ul className="mt-4 grid gap-x-8 sm:grid-cols-2">
              {food.ingredients.map((item, index) => (
                <li key={`${item.name}-${index}`} className="flex items-baseline justify-between gap-4 border-b border-dashed border-border py-2.5 text-[15px]">
                  <span className="min-w-[40%] flex-1">{item.name}</span>
                  <span className="max-w-[60%] text-right font-semibold">{item.amount}</span>
                </li>
              ))}
            </ul>
          </section>

          <section aria-labelledby="steps-title">
            <h2 id="steps-title" className="text-2xl font-bold">
              {t('detail.steps')}
            </h2>
            <ol className="mt-4 space-y-4">
              {food.steps.map((step, index) => (
                <li key={index} className="flex gap-4">
                  <span className="chip-active grid size-9 shrink-0 place-items-center rounded-full text-sm font-bold">{index + 1}</span>
                  <p className="pt-1.5 text-[16px] leading-relaxed">{step}</p>
                </li>
              ))}
            </ol>
          </section>

          {food.tags.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {food.tags.map((tag) => (
                <span key={tag} className="rounded-full bg-surface px-3 py-1.5 text-sm text-ink-muted ring-1 ring-border">
                  #{tag}
                </span>
              ))}
            </div>
          )}
        </div>

        <aside className="space-y-6 lg:sticky lg:top-24 lg:self-start">
          <NutritionCard food={food} />
          <PlacesSection food={food} />
          <RatingCard food={food} />
          {(food.image_credit.author || food.image_credit.license) && (
            <p className="px-2 text-xs leading-relaxed text-ink-soft">
              {t('detail.photo')}: {food.image_credit.author ?? t('detail.unknownAuthor')}
              {food.image_credit.license && ` · ${food.image_credit.license}`}
              {food.image_credit.source_url && (
                <>
                  {' · '}
                  <a href={food.image_credit.source_url} target="_blank" rel="noopener noreferrer" className="underline hover:text-ink">
                    Wikimedia Commons
                  </a>
                </>
              )}
            </p>
          )}
        </aside>
      </div>

      {food.similar.length > 0 && (
        <section aria-labelledby="similar-title" className="container mt-14">
          <h2 id="similar-title" className="text-2xl font-bold">
            {t('detail.similar')}
          </h2>
          <FoodGrid className="mt-5">
            {food.similar.slice(0, 4).map((item) => (
              <FoodCard key={item.id} food={item} />
            ))}
          </FoodGrid>
        </section>
      )}
    </article>
  );
}
