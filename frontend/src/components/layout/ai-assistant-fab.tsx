'use client';

import { Bot, Sparkles } from 'lucide-react';

import { Dialog, DialogContent, DialogDescription, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { useI18n } from '@/i18n/provider';

/**
 * Điểm vào của Trợ lý AI. Tính năng chưa triển khai — nút này chỉ giới thiệu trước và nói rõ là "sắp ra mắt".
 * Khi có backend AI, thay nội dung hộp thoại bằng khung trò chuyện.
 */
export function AiAssistantFab() {
  const { t } = useI18n();
  return (
    <Dialog>
      <DialogTrigger
        className="fixed bottom-5 right-5 z-30 hidden items-center sm:flex gap-2 rounded-full bg-dark py-3 pl-3.5 pr-4 text-sm font-semibold text-white shadow-glass ring-1 ring-white/15 transition hover:-translate-y-0.5 hover:bg-dark/90"
        aria-label={t('ai.aria')}
      >
        <span className="grid size-8 place-items-center rounded-full bg-primary">
          <Bot className="size-[18px]" />
        </span>
        <span className="hidden sm:inline">{t('ai.label')}</span>
      </DialogTrigger>
      <DialogContent className="max-w-md text-center">
        <div className="mx-auto grid size-16 place-items-center rounded-full bg-primary-50 text-primary">
          <Sparkles className="size-8" />
        </div>
        <DialogTitle className="mt-3 pr-0">{t('ai.title')}</DialogTitle>
        <DialogDescription className="mt-2">{t('ai.desc')}</DialogDescription>
      </DialogContent>
    </Dialog>
  );
}
