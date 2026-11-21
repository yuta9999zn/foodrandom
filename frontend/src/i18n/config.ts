// Ngôn ngữ và thị trường. Giữ khớp với backend/src/i18n.rs.

export const LOCALES = ['vi', 'en', 'ja', 'ko', 'zh'] as const;
export type Locale = (typeof LOCALES)[number];
export const DEFAULT_LOCALE: Locale = 'vi';

export const LOCALE_NAMES: Record<Locale, string> = {
  vi: 'Tiếng Việt',
  en: 'English',
  ja: '日本語',
  ko: '한국어',
  zh: '中文',
};

/** Thẻ BCP 47 dùng cho Intl (định dạng số, tiền, ngày). */
export const INTL_LOCALE: Record<Locale, string> = { vi: 'vi-VN', en: 'en-GB', ja: 'ja-JP', ko: 'ko-KR', zh: 'zh-CN' };

export const MARKETS = ['VN', 'JP', 'KR', 'CN', 'GB'] as const;
export type MarketCode = (typeof MARKETS)[number];
export const DEFAULT_MARKET: MarketCode = 'VN';

export interface MarketInfo {
  code: MarketCode;
  currency: 'VND' | 'JPY' | 'KRW' | 'CNY' | 'GBP';
  flag: string;
  /** Ngôn ngữ gợi ý khi người dùng chọn thị trường này lần đầu. */
  language: Locale;
  /** Cận của 5 mức ngân sách b1..b5 (tiền tệ địa phương). */
  bands: [number, number, number, number];
}

export const MARKET_INFO: Record<MarketCode, MarketInfo> = {
  VN: { code: 'VN', currency: 'VND', flag: '🇻🇳', language: 'vi', bands: [30_000, 50_000, 100_000, 200_000] },
  JP: { code: 'JP', currency: 'JPY', flag: '🇯🇵', language: 'ja', bands: [800, 1_500, 3_000, 6_000] },
  KR: { code: 'KR', currency: 'KRW', flag: '🇰🇷', language: 'ko', bands: [8_000, 15_000, 30_000, 60_000] },
  CN: { code: 'CN', currency: 'CNY', flag: '🇨🇳', language: 'zh', bands: [25, 50, 100, 200] },
  GB: { code: 'GB', currency: 'GBP', flag: '🇬🇧', language: 'en', bands: [8, 15, 30, 60] },
};

export const BUDGET_CODES = ['b1', 'b2', 'b3', 'b4', 'b5'] as const;
export type BudgetCode = (typeof BUDGET_CODES)[number];

export const LOCALE_COOKIE = 'fr_lang';
export const MARKET_COOKIE = 'fr_market';

export function isLocale(value: unknown): value is Locale {
  return typeof value === 'string' && (LOCALES as readonly string[]).includes(value);
}

export function isMarket(value: unknown): value is MarketCode {
  return typeof value === 'string' && (MARKETS as readonly string[]).includes(value);
}

/** Đoán ngôn ngữ từ Accept-Language / navigator.language. */
export function guessLocale(header: string | null | undefined): Locale {
  for (const part of (header ?? '').split(',')) {
    const base = part.trim().slice(0, 2).toLowerCase();
    if (isLocale(base)) return base;
  }
  return DEFAULT_LOCALE;
}

/** Đoán thị trường từ múi giờ của trình duyệt (chỉ để gợi ý sẵn trong hộp chọn quốc gia). */
export function guessMarketFromTimeZone(timeZone: string | undefined): MarketCode | null {
  switch (timeZone) {
    case 'Asia/Ho_Chi_Minh':
    case 'Asia/Saigon':
      return 'VN';
    case 'Asia/Tokyo':
      return 'JP';
    case 'Asia/Seoul':
      return 'KR';
    case 'Asia/Shanghai':
    case 'Asia/Chongqing':
    case 'Asia/Urumqi':
    case 'Asia/Harbin':
      return 'CN';
    case 'Europe/London':
      return 'GB';
    default:
      return null;
  }
}
