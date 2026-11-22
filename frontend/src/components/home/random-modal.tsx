'use client';

import { AnimatePresence, motion, useReducedMotion } from 'framer-motion';
import { ArrowRight, Flame, Footprints, MapPin, Shuffle, Sparkles } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';
import { useEffect, useState } from 'react';

import { PriceText, RatingLine, StyleChip, TypeChip } from '@/components/food/food-card';
import { MapsSearchButton } from '@/components/food/maps-search-button';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { useI18n } from '@/i18n/provider';
import type { FoodListItem, RandomResponse } from '@/lib/types';
import { formatDistance } from '@/lib/utils';

export type RandomPhase = 'idle' | 'spinning' | 'result' | 'error';

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  phase: RandomPhase;
  /** Các món của trang đang xem — dùng làm hình chạy trong lúc "quay". Kết quả thật do backend chọn. */
  candidates: FoodListItem[];
  result: RandomResponse | null;
  errorMessage: string | null;
  onAgain: () => void;
}

/** Hiệu ứng "quay số": lướt nhanh qua ảnh các món trong trang. */
function Spinner({ candidates }: { candidates: FoodListItem[] }) {
  const { t } = useI18n();
  const [index, setIndex] = useState(0);
  const reduced = useReducedMotion();

  useEffect(() => {
    if (candidates.length < 2 || reduced) return;
    const timer = setInterval(() => setIndex((i) => (i + 1) % candidates.length), 110);
    return () => clearInterval(timer);
  }, [candidates.length, reduced]);

  const current = candidates[index % Math.max(candidates.length, 1)];

  return (
    <div className="p-6 text-center" aria-live="polite">
      <div className="relative mx-auto aspect-[16/10] w-full overflow-hidden rounded-2xl bg-primary-50">
        {current && <Image src={current.image_url} alt="" fill sizes="480px" className="object-cover blur-[1.5px]" />}
        <div className="absolute inset-0 bg-dark/35" />
        <motion.div
          className="absolute inset-0 grid place-items-center text-white"
          animate={reduced ? undefined : { rotate: [0, -12, 12, -8, 8, 0], scale: [1, 1.12, 1] }}
          transition={{ duration: 0.7, repeat: Infinity }}
        >
          <Shuffle className="size-16 drop-shadow-lg" strokeWidth={2.2} />
        </motion.div>
      </div>
      <p className="mt-5 text-lg font-bold">{t('random.spinning')}</p>
      <p className="mt-1 line-clamp-1 text-[15px] text-ink-muted">{current?.name ?? t('random.wait')}</p>
    </div>
  );
}

function Result({ result, onAgain, onClose }: { result: RandomResponse; onAgain: () => void; onClose: () => void }) {
  const { t } = useI18n();
  const food = result.food;
  const place = food.nearest_place;

  return (
    <motion.div
      key={food.slug}
      initial={{ opacity: 0, scale: 0.9, y: 16 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      transition={{ type: 'spring', stiffness: 260, damping: 22 }}
    >
      <div className="relative aspect-[2/1] w-full overflow-hidden bg-primary-50 sm:aspect-[16/10]">
        <Image src={food.image_url} alt={food.name} fill sizes="(min-width: 640px) 560px, 100vw" className="object-cover" priority />
        <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-black/10 to-transparent" />
        <motion.span
          initial={{ scale: 0, rotate: -20 }}
          animate={{ scale: 1, rotate: 0 }}
          transition={{ delay: 0.15, type: 'spring', stiffness: 300 }}
          className="absolute left-4 top-4 inline-flex items-center gap-1.5 rounded-full bg-primary px-3.5 py-1.5 text-sm font-bold text-white shadow-lg"
        >
          <Sparkles className="size-4 fill-secondary text-secondary" /> {t('random.todayPick')}
        </motion.span>
        {place && (
          <span className="absolute bottom-4 left-4 inline-flex items-center gap-1.5 rounded-full bg-dark/75 px-3 py-1 text-sm font-semibold text-white backdrop-blur">
            <Footprints className="size-4" /> {formatDistance(place.distance_km)}
          </span>
        )}
      </div>

      <div className="p-5 sm:p-6">
        <DialogTitle className="pr-0 text-xl sm:text-2xl">{food.name}</DialogTitle>
        {food.original_name && food.original_name !== food.name && <p className="mt-0.5 text-[15px] text-ink-muted">{food.original_name}</p>}

        <div className="mt-3 flex flex-wrap gap-1.5">
          <StyleChip style={food.style} />
          <TypeChip food={food} />
          <span className="inline-flex items-center rounded-full bg-background px-2.5 py-1 text-[13px] font-medium text-ink-muted">
            {t(`options.cuisines.${food.cuisine}`)}
          </span>
        </div>

        <div className="mt-4 flex flex-wrap items-center gap-x-5 gap-y-1.5">
          <PriceText food={food} className="text-[22px]" />
          <span className="inline-flex items-center gap-1.5 font-semibold">
            <Flame className="size-[18px] text-secondary" /> {t('random.perServing', { kcal: food.calories_kcal })}
          </span>
          <RatingLine food={food} />
        </div>

        <DialogDescription className="mt-3 line-clamp-2 sm:line-clamp-3">{food.description}</DialogDescription>

        {place ? (
          <div className="mt-4 flex items-start gap-3 rounded-2xl bg-background p-3.5">
            <MapPin className="mt-0.5 size-5 shrink-0 text-primary" />
            <div className="min-w-0 text-[15px]">
              <p className="font-semibold">
                {place.name} {place.is_mock && <span className="font-normal text-ink-muted">{t('random.mockData')}</span>}
              </p>
              <p className="text-ink-muted">{place.address ?? t('random.noAddress')}</p>
            </div>
          </div>
        ) : (
          <p className="mt-4 hidden rounded-2xl bg-background px-4 py-3 text-sm text-ink-muted sm:block">{t('random.noLocationHint')}</p>
        )}

        <div className="mt-4 grid grid-cols-1 gap-2 sm:mt-5 sm:grid-cols-3 sm:gap-2.5">
          <Button asChild variant="outline" className="sm:order-1">
            <Link href={`/foods/${food.slug}`} onClick={onClose}>
              {t('random.details')} <ArrowRight />
            </Link>
          </Button>
          <MapsSearchButton variant="soft" className="sm:order-2" name={food.name} />
          <Button className="sm:order-3" onClick={onAgain}>
            <Shuffle /> {t('random.again')}
          </Button>
        </div>

        <p className="mt-3 text-center text-xs text-ink-soft">
          {result.scope === 'page'
            ? t('random.poolPage', { n: result.pool_size, page: result.page })
            : t('random.poolAll', { n: result.pool_size })}
        </p>
      </div>
    </motion.div>
  );
}

/** Màn hình kết quả random: quay → hiện món được backend chọn. */
export function RandomModal({ open, onOpenChange, phase, candidates, result, errorMessage, onAgain }: Props) {
  const { t } = useI18n();
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-[560px] overflow-hidden p-0">
        <AnimatePresence mode="wait" initial={false}>
          {phase === 'spinning' || phase === 'idle' ? (
            <motion.div key="spin" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
              <DialogTitle className="sr-only">{t('random.spinningTitle')}</DialogTitle>
              <DialogDescription className="sr-only">{t('random.spinningDesc')}</DialogDescription>
              <Spinner candidates={candidates} />
            </motion.div>
          ) : phase === 'error' ? (
            <motion.div key="error" initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="p-8 text-center">
              <DialogTitle className="pr-0">{t('random.errorTitle')}</DialogTitle>
              <DialogDescription className="mt-2" role="alert">
                {errorMessage}
              </DialogDescription>
              <Button className="mt-5" onClick={onAgain}>
                <Shuffle /> {t('random.retry')}
              </Button>
            </motion.div>
          ) : result ? (
            <Result key={`result-${result.food.slug}`} result={result} onAgain={onAgain} onClose={() => onOpenChange(false)} />
          ) : null}
        </AnimatePresence>
      </DialogContent>
    </Dialog>
  );
}
