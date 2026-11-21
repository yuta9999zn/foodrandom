'use client';

import { Coins, Dices, MapPin, SlidersHorizontal } from 'lucide-react';
import Image from 'next/image';

import { useI18n } from '@/i18n/provider';
import { cn } from '@/lib/utils';

const ICONS = [MapPin, Coins, SlidersHorizontal, Dices];

/** Tiêu đề viết tay "Hôm nay ăn gì?" dựng bằng HTML (dùng cho điện thoại và các ngôn ngữ khác tiếng Việt). */
function Headline({ className }: { className?: string }) {
  const { t, locale } = useI18n();
  // Font viết tay chỉ có chữ Latin; tiếng Nhật/Hàn/Trung dùng font đậm hệ thống.
  const cjk = locale === 'ja' || locale === 'ko' || locale === 'zh';
  return (
    <p
      className={cn(
        'leading-[0.95] drop-shadow-[0_2px_12px_rgba(0,0,0,0.6)]',
        cjk ? 'font-sans font-bold tracking-tight' : 'font-display font-bold',
        className,
      )}
    >
      {t('hero.line1')}
      <br />
      <span className="relative ml-[0.6em] inline-block">
        {t('hero.line2')}
        <span className="absolute -bottom-[0.08em] left-[0.4em] right-0 h-[0.1em] -rotate-3 rounded-full bg-primary" aria-hidden="true" />
      </span>
    </p>
  );
}

function Highlights({ className }: { className?: string }) {
  const { list } = useI18n();
  return (
    <ul className={cn('flex flex-wrap gap-2', className)}>
      {list('hero.highlights').map((label, i) => {
        const Icon = ICONS[i] ?? MapPin;
        return (
          <li
            key={label}
            className="inline-flex items-center gap-1.5 rounded-full border border-white/20 bg-black/35 px-3 py-1.5 text-[13px] font-medium backdrop-blur"
          >
            <Icon className="size-4 text-secondary" /> {label}
          </li>
        );
      })}
    </ul>
  );
}

/**
 * Banner trang chủ.
 * - Tiếng Việt, từ tablet trở lên: dùng nguyên banner FoodRandom (tiêu đề và phụ đề nằm sẵn trong ảnh).
 * - Ngôn ngữ khác: banner đã xoá chữ, tiêu đề và phụ đề dựng bằng HTML theo ngôn ngữ.
 * - Điện thoại: phần tô mì của banner làm nền, chữ dựng bằng HTML.
 */
export function Hero() {
  const { t, locale } = useI18n();
  const nativeBanner = locale === 'vi';
  const cjk = locale === 'ja' || locale === 'ko' || locale === 'zh';

  return (
    <section className="relative isolate bg-[#120d11]" aria-labelledby="hero-title">
      {/* Tablet / desktop */}
      <div className="relative hidden aspect-[2172/724] w-full md:block">
        <Image
          src={nativeBanner ? '/images/hero/home-banner.jpg' : '/images/hero/home-banner-clean.jpg'}
          alt={t('hero.alt')}
          fill
          priority
          sizes="100vw"
          className="object-cover object-top"
        />
        {!nativeBanner && (
          <div className="absolute left-[11.5%] top-[15%] w-[42%] text-[#fbf1e6]">
            <Headline className={cn(cjk ? 'text-[clamp(36px,4.3vw,84px)] leading-[1.12]' : 'text-[clamp(40px,5.4vw,104px)]')} />
            <p className="mt-[1.4vw] text-[clamp(15px,1.4vw,27px)] font-medium text-white/90 drop-shadow">{t('hero.subtitle')}</p>
          </div>
        )}
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-[46%] bg-gradient-to-b from-transparent via-background/55 to-background" />
      </div>

      {/* Điện thoại */}
      <div className="relative min-h-[430px] md:hidden">
        <Image src="/images/hero/banner-mobile.jpg" alt="" fill priority sizes="100vw" className="object-cover object-[62%_center]" />
        <div className="absolute inset-0 bg-gradient-to-b from-black/70 via-black/45 to-background" />
        <div className="relative px-5 pb-28 pt-24 text-white">
          <Headline className={cjk ? 'text-[44px] leading-[1.12]' : 'text-[54px]'} />
          <p className="mt-5 text-[15px] font-medium text-white/90">{t('hero.subtitle')}</p>
          <Highlights className="mt-4" />
        </div>
      </div>

      <div className="sr-only">
        <h1 id="hero-title">
          {t('hero.line1')} {t('hero.line2')}
        </h1>
        <p>{t('hero.subtitle')}</p>
      </div>
    </section>
  );
}
