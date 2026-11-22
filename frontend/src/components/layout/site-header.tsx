'use client';

import { Bell, Check, Heart, Languages, Menu, Search } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';
import { toast } from 'sonner';

import { Flag } from '@/components/flag';
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { LOCALE_NAMES, LOCALES, MARKET_INFO, MARKETS } from '@/i18n/config';
import { useI18n } from '@/i18n/provider';
import { cn } from '@/lib/utils';

import { Logo } from './logo';
import { SearchDialog } from './search-dialog';

const NAV = [
  { href: '/kham-pha', key: 'nav.explore' },
  { href: '/', key: 'nav.random' },
  { href: '/bo-suu-tap', key: 'nav.collection' },
  { href: '/cong-dong', key: 'nav.community' },
];

const iconButton =
  'grid size-11 place-items-center rounded-full text-white/90 transition hover:bg-white/15 hover:text-white [&_svg]:size-[26px]';

/** Nút đổi ngôn ngữ và quốc gia (tiền tệ) trong một menu. */
function PrefsMenu() {
  const { t, locale, market, setLocale, setMarket } = useI18n();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        className="flex h-11 items-center gap-1.5 rounded-full px-2.5 text-[15px] font-semibold text-white/90 transition hover:bg-white/15 hover:text-white"
        aria-label={`${t('nav.language')} · ${t('nav.country')}`}
      >
        <Languages className="size-[22px]" />
        <span className="hidden items-center gap-1.5 sm:inline-flex">
          {locale.toUpperCase()} <Flag code={market} />
        </span>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-64">
        <DropdownMenuLabel className="font-semibold text-ink">{t('nav.language')}</DropdownMenuLabel>
        {LOCALES.map((code) => (
          <DropdownMenuItem key={code} onSelect={() => code !== locale && setLocale(code)} lang={code}>
            <span className="flex-1">{LOCALE_NAMES[code]}</span>
            {code === locale && <Check className="!text-primary" />}
          </DropdownMenuItem>
        ))}
        <DropdownMenuSeparator />
        <DropdownMenuLabel className="font-semibold text-ink">{t('market.switchTitle')}</DropdownMenuLabel>
        {MARKETS.map((code) => (
          <DropdownMenuItem
            key={code}
            onSelect={() => {
              if (code === market) return;
              setMarket(code);
              toast.success(t('market.changed', { country: t(`market.names.${code}`), currency: MARKET_INFO[code].currency }));
            }}
          >
            <Flag code={code} />
            <span className="flex-1">{t(`market.names.${code}`)}</span>
            <span className="text-xs text-ink-muted">{MARKET_INFO[code].currency}</span>
            {code === market && <Check className="!text-primary" />}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function SiteHeader() {
  const { t } = useI18n();
  const pathname = usePathname();
  const [searchOpen, setSearchOpen] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);

  // Trang chủ: header trong suốt nằm đè lên banner. Trang khác: thanh tối cố định.
  const overlay = pathname === '/';
  const isActive = (href: string) => (href === '/' ? pathname === '/' : pathname.startsWith(href));

  return (
    <header
      className={cn(
        'inset-x-0 top-0 z-40',
        overlay ? 'absolute bg-gradient-to-b from-black/55 via-black/25 to-transparent' : 'sticky bg-dark shadow-lg shadow-dark/20',
      )}
    >
      <div className="container flex h-[60px] items-center gap-1.5 sm:h-[68px] sm:gap-3 lg:h-[72px]">
        <button type="button" className={cn(iconButton, 'lg:hidden')} aria-label={t('nav.openMenu')} onClick={() => setMenuOpen(true)}>
          <Menu />
        </button>

        <Logo className="mr-auto lg:mr-6 xl:mr-10" />

        <nav className="mr-auto hidden items-center gap-1 lg:flex" aria-label="Main">
          {NAV.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              aria-current={isActive(item.href) ? 'page' : undefined}
              className={cn(
                'whitespace-nowrap rounded-full px-3 py-2.5 text-[16px] font-medium text-white/85 transition hover:text-white xl:px-5 xl:text-[17px]',
                isActive(item.href) &&
                  'bg-white/15 font-semibold text-white shadow-[inset_0_-2px_0_#E85D78,inset_0_1px_0_rgba(255,255,255,0.18)] backdrop-blur',
              )}
            >
              {t(item.key)}
            </Link>
          ))}
        </nav>

        <div className="flex shrink-0 items-center gap-0.5 sm:gap-2">
          <button type="button" className={iconButton} aria-label={t('nav.search')} onClick={() => setSearchOpen(true)}>
            <Search />
          </button>

          <PrefsMenu />

          <DropdownMenu>
            <DropdownMenuTrigger className={cn(iconButton, 'hidden xl:grid')} aria-label={t('nav.notifications')}>
              <Bell />
            </DropdownMenuTrigger>
            <DropdownMenuContent className="w-72">
              <DropdownMenuLabel className="font-semibold text-ink">{t('nav.notifications')}</DropdownMenuLabel>
              <DropdownMenuSeparator />
              <p className="px-3 py-6 text-center text-sm text-ink-muted">{t('nav.noNotifications')}</p>
            </DropdownMenuContent>
          </DropdownMenu>

          <Link href="/bo-suu-tap" className={cn(iconButton, 'hidden sm:grid')} aria-label={t('nav.myCollection')} title={t('nav.myCollection')}>
            <Heart />
          </Link>
        </div>
      </div>

      <SearchDialog open={searchOpen} onOpenChange={setSearchOpen} />

      <Dialog open={menuOpen} onOpenChange={setMenuOpen}>
        <DialogContent className="left-0 top-0 h-dvh max-h-dvh w-72 max-w-[85vw] translate-x-0 translate-y-0 rounded-none rounded-r-3xl border-0 bg-dark p-5 text-white data-[state=closed]:slide-out-to-left data-[state=open]:slide-in-from-left data-[state=closed]:zoom-out-100 data-[state=open]:zoom-in-100">
          <DialogTitle className="sr-only">{t('nav.menu')}</DialogTitle>
          <Logo />
          <nav className="mt-8 flex flex-col gap-1" aria-label="Main">
            {NAV.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                onClick={() => setMenuOpen(false)}
                aria-current={isActive(item.href) ? 'page' : undefined}
                className={cn('rounded-2xl px-4 py-3 text-[17px] font-medium text-white/85', isActive(item.href) && 'bg-white/15 font-semibold text-white')}
              >
                {t(item.key)}
              </Link>
            ))}
          </nav>
        </DialogContent>
      </Dialog>
    </header>
  );
}
