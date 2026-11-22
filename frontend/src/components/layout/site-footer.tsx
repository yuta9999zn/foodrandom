'use client';

import Link from 'next/link';

import { useI18n } from '@/i18n/provider';

import { LogoMark } from './logo';

export function SiteFooter() {
  const { t } = useI18n();
  return (
    <footer className="mt-20 bg-dark text-white/75">
      <div className="container grid grid-cols-1 gap-8 py-12 md:grid-cols-[1.4fr_1fr_1fr]">
        <div>
          <div className="flex items-center gap-2.5 text-white">
            <LogoMark className="h-9" />
            <span className="text-2xl font-bold">FoodRandom</span>
          </div>
          <p className="mt-3 max-w-sm text-[15px] leading-relaxed">{t('footer.tagline')}</p>
        </div>
        <nav aria-label={t('footer.explore')}>
          <h2 className="font-semibold text-white">{t('footer.explore')}</h2>
          <ul className="mt-3 space-y-2 text-[15px]">
            <li>
              <Link className="hover:text-white" href="/">
                {t('nav.random')}
              </Link>
            </li>
            <li>
              <Link className="hover:text-white" href="/kham-pha">
                {t('footer.byCountry')}
              </Link>
            </li>
            <li>
              <Link className="hover:text-white" href="/bo-suu-tap">
                {t('nav.collection')}
              </Link>
            </li>
            <li>
              <Link className="hover:text-white" href="/cong-dong">
                {t('nav.community')}
              </Link>
            </li>
          </ul>
        </nav>
        <div>
          <h2 className="font-semibold text-white">{t('footer.sources')}</h2>
          <ul className="mt-3 space-y-2 text-[15px] leading-relaxed">
            <li>{t('footer.sourcePhotos')}</li>
            <li>{t('footer.sourcePlaces')}</li>
            <li>{t('footer.sourcePrices')}</li>
          </ul>
        </div>
      </div>
      <div className="border-t border-white/10 py-5 text-center text-sm text-white/55">© 2026 FoodRandom</div>
    </footer>
  );
}
