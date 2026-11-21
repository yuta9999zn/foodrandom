import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** "0.4 km" hoặc "850 m" */
export function formatDistance(km: number) {
  if (km < 1) return `${Math.max(10, Math.round(km * 100) * 10)} m`;
  return `${km < 10 ? km.toFixed(1) : Math.round(km)} km`;
}

/** "1,200+" — số lớn làm tròn xuống hàng chục kèm dấu "+". */
export function formatCount(count: number, format: (n: number) => string = (n) => String(n)) {
  if (count >= 100) return `${format(Math.floor(count / 10) * 10)}+`;
  return format(count);
}

export function formatMinutes(minutes: number, t: (key: string, vars?: Record<string, string | number>) => string) {
  if (minutes < 60) return t('time.minutes', { n: minutes });
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return rest ? t('time.hoursMinutes', { h: hours, m: rest }) : t('time.hours', { n: hours });
}
