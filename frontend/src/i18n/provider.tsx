'use client';

import { useRouter } from 'next/navigation';
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from 'react';

import { INTL_LOCALE, LOCALE_COOKIE, MARKET_COOKIE, MARKET_INFO, type Locale, type MarketCode } from './config';
import { translate, type Messages } from './index';

interface I18nContextValue {
  locale: Locale;
  market: MarketCode;
  /** `false` khi người dùng chưa từng chọn quốc gia (hiện hộp chọn quốc gia). */
  marketChosen: boolean;
  messages: Messages;
  t: (key: string, vars?: Record<string, string | number>) => string;
  /** Mảng chuỗi trong từ điển (ví dụ hero.highlights). */
  list: (key: string) => string[];
  setLocale: (locale: Locale) => void;
  setMarket: (market: MarketCode) => void;
  formatPrice: (min: number | null, max: number | null) => string | null;
  formatMoney: (value: number) => string;
  formatNumber: (value: number) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

function writeCookie(name: string, value: string) {
  document.cookie = `${name}=${value}; path=/; max-age=${60 * 60 * 24 * 365}; samesite=lax`;
}

export function I18nProvider({
  locale,
  market: initialMarket,
  marketChosen: initialChosen,
  messages,
  children,
}: {
  locale: Locale;
  market: MarketCode;
  marketChosen: boolean;
  messages: Messages;
  children: ReactNode;
}) {
  const router = useRouter();
  const [market, setMarketState] = useState(initialMarket);
  const [marketChosen, setMarketChosen] = useState(initialChosen);

  const setLocale = useCallback(
    (next: Locale) => {
      writeCookie(LOCALE_COOKIE, next);
      // Từ điển và nội dung món do máy chủ dựng lại theo ngôn ngữ mới.
      router.refresh();
    },
    [router],
  );

  const setMarket = useCallback(
    (next: MarketCode) => {
      writeCookie(MARKET_COOKIE, next);
      setMarketState(next);
      setMarketChosen(true);
      router.refresh();
    },
    [router],
  );

  const value = useMemo<I18nContextValue>(() => {
    const intl = INTL_LOCALE[locale];
    const currency = MARKET_INFO[market].currency;
    const money = new Intl.NumberFormat(intl, {
      style: 'currency',
      currency,
      maximumFractionDigits: currency === 'GBP' || currency === 'CNY' ? 2 : 0,
      minimumFractionDigits: 0,
    });
    const number = new Intl.NumberFormat(intl);
    const formatMoney = (v: number) => money.format(v);
    return {
      locale,
      market,
      marketChosen,
      messages,
      t: (key, vars) => translate(messages, key, vars),
      list: (key) => {
        let node: unknown = messages;
        for (const part of key.split('.')) node = (node as Record<string, unknown> | undefined)?.[part];
        return Array.isArray(node) ? (node as string[]) : [];
      },
      setLocale,
      setMarket,
      formatMoney,
      formatNumber: (v) => number.format(v),
      formatPrice: (min, max) => {
        if (min == null || max == null) return null;
        return min === max ? formatMoney(min) : `${formatMoney(min)} – ${formatMoney(max)}`;
      },
    };
  }, [locale, market, marketChosen, messages, setLocale, setMarket]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) throw new Error('useI18n phải dùng bên trong I18nProvider');
  return context;
}

/** Viết tắt thường dùng. */
export function useT() {
  return useI18n().t;
}
