import Link from 'next/link';

import { cn } from '@/lib/utils';

/** Biểu tượng bát FoodRandom: viền trắng, lòng bát hồng, tia lấp lánh vàng. */
export function LogoMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 56 48" fill="none" aria-hidden="true" className={cn('h-10 w-auto', className)}>
      <path
        d="M6 19.5c0 12.4 9.6 22 22 22s22-9.6 22-22"
        stroke="currentColor"
        strokeWidth="3.2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path d="M11.5 23.5c1.6 7.6 8.2 13 16.5 13s14.9-5.4 16.5-13c-5 2.6-10.6 3.8-16.5 3.8s-11.5-1.2-16.5-3.8Z" fill="#E85D78" />
      <ellipse cx="28" cy="19.5" rx="23" ry="5.6" stroke="currentColor" strokeWidth="3.2" />
      <path d="M26 16.5c2.4-3 .4-6 3.2-9.4M32.5 5.8l1.6-3" stroke="currentColor" strokeWidth="2.8" strokeLinecap="round" />
      <circle cx="38.6" cy="5" r="1.9" fill="#F4C542" />
      <path d="M21.5 10.2 19 8.4" stroke="#F4A261" strokeWidth="2.4" strokeLinecap="round" />
    </svg>
  );
}

export function Logo({ className }: { className?: string }) {
  return (
    <Link href="/" className={cn('flex items-center gap-2.5 text-white', className)} aria-label="FoodRandom — trang chủ">
      <LogoMark />
      <span className="text-[26px] font-bold leading-none tracking-tight">FoodRandom</span>
    </Link>
  );
}
