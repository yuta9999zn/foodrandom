import { CircleAlert, RotateCw, SearchX } from 'lucide-react';
import type { ReactNode } from 'react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

/** Trạng thái rỗng dùng chung. */
export function EmptyState({
  title,
  description,
  action,
  icon,
  className,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
  icon?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        'flex flex-col items-center rounded-3xl border border-dashed border-border bg-surface/70 px-6 py-14 text-center',
        className,
      )}
    >
      <div className="grid size-16 place-items-center rounded-full bg-primary-50 text-primary">{icon ?? <SearchX className="size-8" />}</div>
      <h3 className="mt-4 text-lg font-bold">{title}</h3>
      {description && <p className="mt-1.5 max-w-md text-[15px] leading-relaxed text-ink-muted">{description}</p>}
      {action && <div className="mt-5">{action}</div>}
    </div>
  );
}

/** Trạng thái lỗi dùng chung, kèm nút thử lại. */
export function ErrorState({
  title,
  message,
  onRetry,
  retryLabel = 'Thử lại',
  className,
}: {
  title: string;
  message: string;
  onRetry?: () => void;
  retryLabel?: string;
  className?: string;
}) {
  return (
    <div
      role="alert"
      className={cn('flex flex-col items-center rounded-3xl border border-primary-200 bg-primary-50/70 px-6 py-12 text-center', className)}
    >
      <div className="grid size-16 place-items-center rounded-full bg-white text-primary-600">
        <CircleAlert className="size-8" />
      </div>
      <h3 className="mt-4 text-lg font-bold">{title}</h3>
      <p className="mt-1.5 max-w-md text-[15px] leading-relaxed text-ink-muted">{message}</p>
      {onRetry && (
        <Button variant="outline" className="mt-5" onClick={onRetry}>
          <RotateCw /> {retryLabel}
        </Button>
      )}
    </div>
  );
}
