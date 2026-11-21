import type { Metadata, Viewport } from 'next';
import { Be_Vietnam_Pro, Dancing_Script } from 'next/font/google';
import type { ReactNode } from 'react';

import { AiAssistantFab } from '@/components/layout/ai-assistant-fab';
import { MarketPicker } from '@/components/layout/market-picker';
import { SiteFooter } from '@/components/layout/site-footer';
import { SiteHeader } from '@/components/layout/site-header';
import { NearbyDialog } from '@/components/location/nearby-dialog';
import { loadMessages } from '@/i18n';
import { I18nProvider } from '@/i18n/provider';
import { getRequestPrefs } from '@/i18n/server';

import './globals.css';
import { Providers } from './providers';

const sans = Be_Vietnam_Pro({
  subsets: ['latin', 'vietnamese'],
  weight: ['400', '500', '600', '700'],
  variable: '--font-sans',
  display: 'swap',
});

const display = Dancing_Script({
  subsets: ['latin', 'vietnamese'],
  weight: ['700'],
  variable: '--font-display',
  display: 'swap',
});

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || 'http://localhost:3100';

export async function generateMetadata(): Promise<Metadata> {
  const { locale } = await getRequestPrefs();
  const m = await loadMessages(locale);
  return {
    metadataBase: new URL(siteUrl),
    title: { default: m.meta.title, template: '%s · FoodRandom' },
    description: m.meta.description,
    openGraph: { title: m.meta.title, description: m.hero.subtitle, images: ['/images/hero/home-banner.jpg'], type: 'website' },
  };
}

export const viewport: Viewport = {
  themeColor: '#292124',
  width: 'device-width',
  initialScale: 1,
};

export default async function RootLayout({ children }: { children: ReactNode }) {
  const { locale, market, marketChosen } = await getRequestPrefs();
  const messages = await loadMessages(locale);

  return (
    <html lang={locale} className={`${sans.variable} ${display.variable}`}>
      <body className="min-h-dvh font-sans">
        <I18nProvider locale={locale} market={market} marketChosen={marketChosen} messages={messages}>
          <Providers>
            <a
              href="#noi-dung"
              className="sr-only z-50 rounded-full bg-primary px-4 py-2 font-semibold text-white focus:not-sr-only focus:fixed focus:left-4 focus:top-4"
            >
              {messages.nav.skip}
            </a>
            <SiteHeader />
            <main id="noi-dung">{children}</main>
            <SiteFooter />
            <NearbyDialog />
            <MarketPicker />
            <AiAssistantFab />
          </Providers>
        </I18nProvider>
      </body>
    </html>
  );
}
