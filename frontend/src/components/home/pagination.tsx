'use client';

import { ChevronLeft, ChevronRight } from 'lucide-react';

import { useI18n } from '@/i18n/provider';
import { cn } from '@/lib/utils';

/** Dãy số trang rút gọn: 1 … 4 5 6 … 12 */
function pageList(current: number, total: number): (number | 'gap')[] {
  if (total <= 7) return Array.from({ length: total }, (_, i) => i + 1);
  const pages = new Set([1, total, current - 1, current, current + 1]);
  const sorted = [...pages].filter((p) => p >= 1 && p <= total).sort((a, b) => a - b);
  const out: (number | 'gap')[] = [];
  sorted.forEach((page, i) => {
    const previous = sorted[i - 1];
    if (previous !== undefined && page - previous > 1) out.push('gap');
    out.push(page);
  });
  return out;
}

export function Pagination({ page, totalPages, onChange }: { page: number; totalPages: number; onChange: (page: number) => void }) {
  const { t } = useI18n();
  if (totalPages <= 1) return null;
  const base =
    'grid h-11 min-w-11 place-items-center rounded-full px-3 text-[15px] font-semibold transition disabled:pointer-events-none disabled:opacity-40';

  return (
    <nav aria-label={t('results.pagination')} className="mt-9 flex flex-wrap items-center justify-center gap-2">
      <button
        type="button"
        className={cn(base, 'bg-surface text-ink ring-1 ring-border hover:bg-primary-50')}
        onClick={() => onChange(page - 1)}
        disabled={page <= 1}
        aria-label={t('results.prev')}
      >
        <ChevronLeft className="size-5" />
      </button>
      {pageList(page, totalPages).map((item, index) =>
        item === 'gap' ? (
          <span key={`gap-${index}`} className="px-1 text-ink-soft" aria-hidden="true">
            …
          </span>
        ) : (
          <button
            key={item}
            type="button"
            aria-current={item === page ? 'page' : undefined}
            aria-label={t('results.page', { n: item })}
            onClick={() => onChange(item)}
            className={cn(base, item === page ? 'chip-active' : 'bg-surface text-ink ring-1 ring-border hover:bg-primary-50')}
          >
            {item}
          </button>
        ),
      )}
      <button
        type="button"
        className={cn(base, 'bg-surface text-ink ring-1 ring-border hover:bg-primary-50')}
        onClick={() => onChange(page + 1)}
        disabled={page >= totalPages}
        aria-label={t('results.next')}
      >
        <ChevronRight className="size-5" />
      </button>
    </nav>
  );
}
