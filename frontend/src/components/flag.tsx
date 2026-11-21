import type { MarketCode } from '@/i18n/config';
import { cn } from '@/lib/utils';

/**
 * Cờ quốc gia vẽ bằng SVG (emoji cờ không hiển thị trên Windows).
 * Đơn giản hoá ở kích thước nhỏ nhưng vẫn nhận ra được.
 */
export function Flag({ code, className }: { code: MarketCode; className?: string }) {
  const common = cn('inline-block h-[0.95em] w-[1.4em] shrink-0 rounded-[2px] align-[-0.12em] shadow-[0_0_0_1px_rgba(0,0,0,0.12)]', className);
  switch (code) {
    case 'VN':
      return (
        <svg viewBox="0 0 30 20" className={common} aria-hidden="true">
          <rect width="30" height="20" fill="#DA251D" />
          <path d="M15 4.2l1.35 4.15h4.36l-3.53 2.56 1.35 4.15L15 12.5l-3.53 2.56 1.35-4.15-3.53-2.56h4.36z" fill="#FFFF00" />
        </svg>
      );
    case 'JP':
      return (
        <svg viewBox="0 0 30 20" className={common} aria-hidden="true">
          <rect width="30" height="20" fill="#fff" />
          <circle cx="15" cy="10" r="6" fill="#BC002D" />
        </svg>
      );
    case 'KR':
      return (
        <svg viewBox="0 0 30 20" className={common} aria-hidden="true">
          <rect width="30" height="20" fill="#fff" />
          <path d="M15 5a5 5 0 0 1 0 10a2.5 2.5 0 0 1 0-5a2.5 2.5 0 0 0 0-5z" fill="#CD2E3A" />
          <path d="M15 15a5 5 0 0 1 0-10a2.5 2.5 0 0 1 0 5a2.5 2.5 0 0 0 0 5z" fill="#0047A0" />
          <g stroke="#000" strokeWidth="1">
            <path d="M5 4.5l2.5-2M5.8 5.3l2.5-2M22 2.5l2.5 2M21.2 3.3l2.5 2M5 15.5l2.5 2M5.8 14.7l2.5 2M22 17.5l2.5-2M21.2 16.7l2.5-2" />
          </g>
        </svg>
      );
    case 'CN':
      return (
        <svg viewBox="0 0 30 20" className={common} aria-hidden="true">
          <rect width="30" height="20" fill="#DE2910" />
          <path d="M6 3l.9 2.75h2.9l-2.35 1.7.9 2.75L6 8.5 3.65 10.2l.9-2.75L2.2 5.75h2.9z" fill="#FFDE00" />
          <g fill="#FFDE00">
            <circle cx="11" cy="2.5" r="0.8" />
            <circle cx="12.5" cy="4.5" r="0.8" />
            <circle cx="12.5" cy="7" r="0.8" />
            <circle cx="11" cy="9" r="0.8" />
          </g>
        </svg>
      );
    case 'GB':
      return (
        <svg viewBox="0 0 60 30" className={common} aria-hidden="true">
          <rect width="60" height="30" fill="#012169" />
          <path d="M0 0l60 30M60 0L0 30" stroke="#fff" strokeWidth="6" />
          <path d="M0 0l60 30M60 0L0 30" stroke="#C8102E" strokeWidth="2.5" />
          <path d="M30 0v30M0 15h60" stroke="#fff" strokeWidth="10" />
          <path d="M30 0v30M0 15h60" stroke="#C8102E" strokeWidth="6" />
        </svg>
      );
  }
}
