import { cookies, headers } from 'next/headers';

import { DEFAULT_MARKET, guessLocale, isLocale, isMarket, LOCALE_COOKIE, MARKET_COOKIE, type Locale, type MarketCode } from './config';

/** Ngôn ngữ và thị trường của request hiện tại (cookie; chưa có thì đoán ngôn ngữ từ Accept-Language). */
export async function getRequestPrefs(): Promise<{ locale: Locale; market: MarketCode; marketChosen: boolean }> {
  const store = await cookies();
  const cookieLocale = store.get(LOCALE_COOKIE)?.value;
  const cookieMarket = store.get(MARKET_COOKIE)?.value;
  const locale = isLocale(cookieLocale) ? cookieLocale : guessLocale((await headers()).get('accept-language'));
  const market = isMarket(cookieMarket) ? cookieMarket : DEFAULT_MARKET;
  return { locale, market, marketChosen: isMarket(cookieMarket) };
}
