'use client';

import { useMutation } from '@tanstack/react-query';
import {
  Coins,
  Cookie,
  FlaskConical,
  Globe,
  Leaf,
  Loader2,
  LocateFixed,
  MapPin,
  Moon,
  Navigation,
  RotateCcw,
  Shuffle,
  Soup,
  Sparkles,
  Sun,
  Sunrise,
  UtensilsCrossed,
  X,
  type LucideIcon,
} from 'lucide-react';
import { useEffect, useRef, useState, type FormEvent, type ReactNode } from 'react';
import { toast } from 'sonner';

import { Flag } from '@/components/flag';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { usePrefs } from '@/hooks/use-favorites';
import { useGeolocation } from '@/hooks/use-geolocation';
import { BUDGET_CODES, MARKET_INFO, MARKETS, type MarketCode } from '@/i18n/config';
import { useI18n } from '@/i18n/provider';
import { localizedError, locationApi } from '@/lib/api';
import { DISH_TYPES, MEALS, SOUP_TYPES, STYLES } from '@/lib/constants';
import type { Budget, DishType, Meal, SoupType, Style } from '@/lib/types';
import { cn } from '@/lib/utils';
import { activeFilterCount, useFilterStore } from '@/stores/filter-store';
import { MAX_RADIUS_KM, RADIUS_PRESETS, useLocationStore } from '@/stores/location-store';

const MEAL_ICONS: Record<Meal, LucideIcon> = { breakfast: Sunrise, lunch: Sun, dinner: Moon, snack: Cookie };
const STYLE_ICONS: Record<Style, LucideIcon> = { traditional: Leaf, modern: FlaskConical };

function Group({ icon: Icon, label, aside, children, className }: { icon: LucideIcon; label: string; aside?: ReactNode; children: ReactNode; className?: string }) {
  return (
    <div role="group" aria-label={label} className={cn('min-w-0', className)}>
      <div className="mb-3 flex min-h-7 flex-wrap items-center justify-between gap-x-3 gap-y-2">
        <h3 className="flex items-center gap-2.5 whitespace-nowrap text-[17px] font-semibold text-white">
          <Icon className="size-[22px]" strokeWidth={2} /> {label}
        </h3>
        {aside}
      </div>
      {children}
    </div>
  );
}

/** Nhóm nút chọn dạng "segmented" của thiết kế: nền kính, mục đang chọn tô hồng. */
function Segmented({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn('glass-field flex h-11 items-stretch gap-0.5 rounded-xl p-[3px]', className)}>{children}</div>;
}

function SegmentButton({ active, onClick, children, className }: { active: boolean; onClick: () => void; children: ReactNode; className?: string }) {
  return (
    <button
      type="button"
      aria-pressed={active}
      onClick={onClick}
      className={cn(
        'flex min-w-0 flex-1 items-center justify-center gap-1.5 whitespace-nowrap rounded-[10px] px-2 text-[15px] font-medium transition focus-visible:ring-offset-0',
        active ? 'chip-active font-semibold' : 'text-white/90 hover:bg-white/10 hover:text-white',
        className,
      )}
    >
      {children}
    </button>
  );
}

/** Ô "Địa điểm gần bạn": dùng định vị của trình duyệt hoặc gõ địa chỉ rồi Enter. */
function LocationField() {
  const { location, setLocation, clearLocation } = useLocationStore();
  const { locate, locating, geoErrorMessage } = useGeolocation();
  const { t, locale, market } = useI18n();
  const prefs = usePrefs();
  const [text, setText] = useState('');
  const [editing, setEditing] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const boxRef = useRef<HTMLDivElement>(null);
  const geocode = useMutation({ mutationFn: (q: string) => locationApi.geocode(q, prefs) });

  // Bấm ra ngoài thì đóng danh sách gợi ý địa chỉ.
  useEffect(() => {
    const onPointerDown = (event: PointerEvent) => {
      if (!boxRef.current?.contains(event.target as Node)) geocode.reset();
    };
    document.addEventListener('pointerdown', onPointerDown);
    return () => document.removeEventListener('pointerdown', onPointerDown);
  }, [geocode]);

  const handleLocate = async () => {
    try {
      const found = await locate();
      setLocation({ ...found, market });
      setEditing(false);
      setText('');
      toast.success(t('filter.gotLocation', { label: found.label }));
    } catch (error) {
      // Không có quyền vị trí: chuyển sang nhập địa chỉ thủ công.
      toast.warning(geoErrorMessage(error));
      setEditing(true);
      inputRef.current?.focus();
    }
  };

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();
    const q = text.trim();
    if (q.length < 2) {
      toast.error(t('filter.minChars'));
      inputRef.current?.focus();
      return;
    }
    geocode.mutate(q, { onError: (error) => toast.error(localizedError(error, t, locale)) });
  };

  const showLabel = !!location && !editing;

  return (
    <div ref={boxRef} className="relative">
      <form onSubmit={handleSubmit} className="glass-field flex h-11 items-center rounded-xl pl-1 pr-1">
        <button
          type="button"
          onClick={handleLocate}
          disabled={locating}
          aria-label={t('filter.useMyLocation')}
          title={t('filter.useMyLocation')}
          className="grid size-9 shrink-0 place-items-center rounded-lg text-white transition hover:bg-white/15 focus-visible:ring-offset-0"
        >
          {locating ? <Loader2 className="size-5 animate-spin" /> : <LocateFixed className="size-[22px]" />}
        </button>
        <input
          ref={inputRef}
          value={showLabel ? location.label : text}
          readOnly={showLabel}
          onFocus={() => {
            if (showLabel) {
              setEditing(true);
              setText('');
            }
          }}
          onChange={(e) => setText(e.target.value)}
          placeholder={t('filter.locationPlaceholder', { country: t(`market.names.${market}`) })}
          aria-label={t('filter.location')}
          maxLength={200}
          className="h-full min-w-0 flex-1 truncate bg-transparent px-2 text-[15px] font-medium text-white outline-none placeholder:font-normal placeholder:text-white/60 focus-visible:ring-0"
        />
        {showLabel ? (
          <button
            type="button"
            onClick={() => {
              clearLocation();
              setText('');
            }}
            aria-label={t('filter.removeLocation')}
            title={t('filter.removeLocation')}
            className="grid size-9 shrink-0 place-items-center rounded-lg text-white/80 transition hover:bg-white/15 hover:text-white focus-visible:ring-offset-0"
          >
            <X className="size-[18px]" />
          </button>
        ) : (
          <button
            type="submit"
            disabled={geocode.isPending}
            aria-label={t('filter.findAddress')}
            title={t('filter.findAddress')}
            className="grid size-9 shrink-0 place-items-center rounded-lg text-white transition hover:bg-white/15 focus-visible:ring-offset-0"
          >
            {geocode.isPending ? <Loader2 className="size-5 animate-spin" /> : <Navigation className="size-5 fill-white" />}
          </button>
        )}
      </form>

      {geocode.data && (
        <div className="absolute inset-x-0 top-full z-30 mt-2 overflow-hidden rounded-2xl border border-border bg-surface text-ink shadow-card">
          {geocode.data.is_mock && <p className="bg-secondary/25 px-3.5 py-2 text-sm font-semibold">{t('filter.mockAddress')}</p>}
          {geocode.data.results.length === 0 ? (
            <p className="px-3.5 py-3 text-sm text-ink-muted">{t('filter.addressNotFound')}</p>
          ) : (
            <ul className="max-h-64 divide-y divide-border overflow-y-auto">
              {geocode.data.results.map((point) => (
                <li key={`${point.lat},${point.lng}`}>
                  <button
                    type="button"
                    onClick={() => {
                      setLocation({ lat: point.lat, lng: point.lng, label: point.label, source: 'manual', market });
                      setEditing(false);
                      setText('');
                      geocode.reset();
                    }}
                    className="flex w-full items-start gap-2.5 px-3.5 py-3 text-left text-[15px] transition hover:bg-primary-50"
                  >
                    <MapPin className="mt-0.5 size-[18px] shrink-0 text-primary" />
                    <span>{point.label}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

const ALL = 'all';

/** Các mức bán kính trong ô chọn; giá trị tuỳ chỉnh (chọn bằng thanh kéo) cũng được hiện để ô chọn không bị trống. */
function radiusOptions(current: number | null): number[] {
  const values = new Set<number>([...RADIUS_PRESETS, MAX_RADIUS_KM]);
  if (current != null) values.add(current);
  return [...values].sort((a, b) => a - b);
}

/** Nhãn ngắn của mức ngân sách theo tiền tệ thị trường: "Dưới 30k", "¥800 – ¥1,500", "£60+". */
function useBudgetLabels() {
  const { t, market, formatMoney, formatNumber } = useI18n();
  const vn = market === 'VN';
  // "30k – 50k" cho VND; nơi khác ký hiệu tiền chỉ ở số đầu cho gọn: "¥800–1,500".
  const money = (v: number) => (vn ? `${v / 1000}k` : formatMoney(v));
  const plain = (v: number) => (vn ? `${v / 1000}k` : formatNumber(v));
  const range = (lo: number, hi: number) => (vn ? `${money(lo)} – ${money(hi)}` : `${money(lo)}–${plain(hi)}`);
  const [a, b, c, d] = MARKET_INFO[market].bands;
  return {
    b1: t('filter.budgetUnder', { v: money(a) }),
    b2: range(a, b),
    b3: range(b, c),
    b4: range(c, d),
    b5: t('filter.budgetOver', { v: money(d) }),
  } satisfies Record<Budget, string>;
}

function MarketSelect() {
  const { t, market, setMarket } = useI18n();
  return (
    <Select value={market} onValueChange={(v) => setMarket(v as MarketCode)}>
      <SelectTrigger tone="glass" aria-label={t('market.switchTitle')} className="h-8 w-auto gap-1.5 rounded-lg px-2.5 text-[13px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent align="end">
        {MARKETS.map((code) => (
          <SelectItem key={code} value={code}>
            <span className="inline-flex items-center gap-2">
              <Flag code={code} /> {t(`market.names.${code}`)} · {MARKET_INFO[code].currency}
            </span>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

/** Panel bộ lọc — chức năng trung tâm của trang chủ. */
export function FilterPanel({ onRandom, randomDisabled, randomLoading }: { onRandom: () => void; randomDisabled?: boolean; randomLoading?: boolean }) {
  const filter = useFilterStore();
  const { radius, setRadius } = useLocationStore();
  const { t } = useI18n();
  const budgetLabels = useBudgetLabels();
  const activeCount = activeFilterCount(filter);

  const divider = 'lg:border-l lg:border-white/15 lg:pl-7';

  return (
    <section aria-label={t('filter.aria')} className="glass-dark rounded-3xl px-5 py-6 sm:px-7 lg:px-10 lg:py-7">
      <div className="grid grid-cols-1 gap-x-7 gap-y-6 sm:grid-cols-2 lg:grid-cols-[1.42fr_0.82fr_1.34fr_1fr]">
        <Group icon={Soup} label={t('filter.meals')}>
          <Segmented>
            {MEALS.map((value) => {
              const Icon = MEAL_ICONS[value];
              return (
                <SegmentButton
                  key={value}
                  active={filter.meals.includes(value)}
                  onClick={() => filter.toggleMeal(value)}
                  className="flex-col gap-0.5 px-0.5 text-[12px] leading-none tracking-tight 2xl:flex-row 2xl:gap-1.5 2xl:px-2 2xl:text-[15px] 2xl:tracking-normal"
                >
                  <Icon className="size-4 shrink-0 2xl:size-[18px]" /> <span className="max-w-full truncate">{t(`options.meals.${value}`)}</span>
                </SegmentButton>
              );
            })}
          </Segmented>
        </Group>

        <Group icon={UtensilsCrossed} label={t('filter.dishType')}>
          <Select value={filter.dishType ?? ALL} onValueChange={(v) => filter.setDishType(v === ALL ? null : (v as DishType))}>
            <SelectTrigger tone="glass" aria-label={t('filter.dishType')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL}>{t('filter.all')}</SelectItem>
              {DISH_TYPES.map((value) => (
                <SelectItem key={value} value={value}>
                  {t(`options.dishTypes.${value}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Group>

        <Group icon={Globe} label={t('filter.style')} className={divider}>
          <div className="grid h-11 grid-cols-2 gap-1.5 2xl:gap-2">
            {STYLES.map((value) => {
              const Icon = STYLE_ICONS[value];
              const active = filter.style === value;
              return (
                <button
                  key={value}
                  type="button"
                  aria-pressed={active}
                  onClick={() => filter.toggleStyle(value)}
                  className={cn(
                    'flex min-w-0 items-center justify-center gap-1.5 rounded-xl px-1.5 text-[14px] font-medium transition focus-visible:ring-offset-0 sm:text-[15px] lg:text-[14px] 2xl:gap-2 2xl:px-2 2xl:text-[15px]',
                    active ? 'chip-active font-semibold' : 'glass-field text-white/90 hover:bg-white/15 hover:text-white',
                  )}
                >
                  <Icon className="size-[18px] shrink-0 lg:hidden 2xl:block" /> <span className="truncate">{t(`options.styles.${value}`)}</span>
                </button>
              );
            })}
          </div>
        </Group>

        <Group icon={Soup} label={t('filter.soup')} className={divider}>
          <Select value={filter.soupType ?? ALL} onValueChange={(v) => filter.setSoupType(v === ALL ? null : (v as SoupType))}>
            <SelectTrigger tone="glass" aria-label={t('filter.soup')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL}>{t('filter.allSoup')}</SelectItem>
              {SOUP_TYPES.map((value) => (
                <SelectItem key={value} value={value}>
                  {t(`options.soups.${value}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Group>
      </div>

      <div className="my-6 h-px bg-white/15" />

      <div className="grid grid-cols-1 items-end gap-x-7 gap-y-6 lg:grid-cols-[1.42fr_2.16fr_1.3fr]">
        <Group
          icon={MapPin}
          label={t('filter.location')}
          aside={
            <div className="flex items-center gap-1.5">
              <MarketSelect />
              <Select value={radius == null ? 'none' : String(radius)} onValueChange={(v) => setRadius(v === 'none' ? null : Number(v))}>
                <SelectTrigger tone="glass" aria-label={t('filter.radius')} className="h-8 w-auto gap-1.5 rounded-lg px-2.5 text-[13px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent align="end">
                  {radiusOptions(radius).map((km) => (
                    <SelectItem key={km} value={String(km)}>
                      {t('options.radius.within', { km })}
                    </SelectItem>
                  ))}
                  <SelectItem value="none">{t('options.radius.none')}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          }
        >
          <LocationField />
        </Group>

        <Group icon={Coins} label={t('filter.budget')} className={divider}>
          <Segmented className="h-auto flex-wrap sm:h-11 sm:flex-nowrap sm:overflow-x-auto sm:scrollbar-none">
            {BUDGET_CODES.map((value) => (
              <SegmentButton
                key={value}
                active={filter.budgets.includes(value)}
                onClick={() => filter.toggleBudget(value)}
                className="h-[38px] min-w-[30%] px-2.5 text-[14px] sm:h-auto sm:min-w-fit"
              >
                <span>{budgetLabels[value]}</span>
              </SegmentButton>
            ))}
          </Segmented>
        </Group>

        <div className={cn('flex flex-col gap-2', divider)}>
          <button
            type="button"
            onClick={onRandom}
            disabled={randomDisabled}
            className="btn-cta flex h-[60px] w-full items-center justify-center gap-2.5 whitespace-nowrap rounded-2xl px-4 text-[17px] xl:text-[19px] font-bold text-white transition hover:brightness-105 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:ring-offset-0"
          >
            {randomLoading ? <Loader2 className="size-6 animate-spin" /> : <Shuffle className="size-6" strokeWidth={2.4} />}
            {t('filter.random')}
            <Sparkles className="size-5 fill-secondary text-secondary" />
          </button>
          {activeCount > 0 && (
            <button
              type="button"
              onClick={filter.reset}
              className="mx-auto flex items-center gap-1.5 rounded-full px-3 py-1 text-sm font-medium text-white/80 transition hover:text-white focus-visible:ring-offset-0"
            >
              <RotateCcw className="size-3.5" /> {t('filter.clear', { n: activeCount })}
            </button>
          )}
        </div>
      </div>
    </section>
  );
}
