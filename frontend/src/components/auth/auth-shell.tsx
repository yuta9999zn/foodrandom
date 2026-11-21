import Image from 'next/image';
import type { ReactNode } from 'react';

/** Khung chung cho các trang đăng nhập / đăng ký: ảnh banner bên trái, biểu mẫu bên phải. */
export function AuthShell({ title, subtitle, children }: { title: string; subtitle: string; children: ReactNode }) {
  return (
    <div className="container py-10 lg:py-16">
      <div className="mx-auto grid max-w-5xl overflow-hidden rounded-3xl bg-surface shadow-card ring-1 ring-border lg:grid-cols-[1fr_1.05fr]">
        <div className="relative hidden min-h-[520px] lg:block">
          <Image src="/images/hero/banner-mobile.jpg" alt="" fill sizes="520px" className="object-cover" />
          <div className="absolute inset-0 bg-gradient-to-t from-dark/80 via-dark/20 to-transparent" />
        </div>
        <div className="p-6 sm:p-10">
          <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
          <p className="mt-2 text-[15px] leading-relaxed text-ink-muted">{subtitle}</p>
          <div className="mt-7">{children}</div>
        </div>
      </div>
    </div>
  );
}
