'use client';

import { Check, Globe2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { Flag } from '@/components/flag';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { guessMarketFromTimeZone, MARKET_INFO, MARKETS, type MarketCode } from '@/i18n/config';
import { useI18n } from '@/i18n/provider';
import { cn } from '@/lib/utils';

/**
 * Hộp chọn quốc gia hiện ở lần truy cập đầu: người dùng chọn nơi mình đang ở trước,
 * để giá, mức ngân sách và việc tìm quán đều theo đúng quốc gia đó.
 */
export function MarketPicker() {
  const { t, market, marketChosen, setMarket } = useI18n();
  const [open, setOpen] = useState(false);
  const [selected, setSelected] = useState<MarketCode>(market);
  const [suggested, setSuggested] = useState<MarketCode | null>(null);

  useEffect(() => {
    if (marketChosen) return;
    const guess = guessMarketFromTimeZone(Intl.DateTimeFormat().resolvedOptions().timeZone);
    if (guess) {
      setSuggested(guess);
      setSelected(guess);
    }
    setOpen(true);
  }, [marketChosen]);

  const confirm = () => {
    setMarket(selected);
    setOpen(false);
    toast.success(t('market.changed', { country: t(`market.names.${selected}`), currency: MARKET_INFO[selected].currency }));
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        // Đóng mà chưa chọn: dùng lựa chọn đang được gợi ý.
        if (!next && !marketChosen) setMarket(selected);
        setOpen(next);
      }}
    >
      <DialogContent className="max-w-lg">
        <div className="mx-auto grid size-14 place-items-center rounded-full bg-primary-50 text-primary">
          <Globe2 className="size-7" />
        </div>
        <DialogTitle className="mt-3 pr-0 text-center text-2xl">{t('market.title')}</DialogTitle>
        <DialogDescription className="mt-2 text-center">{t('market.description')}</DialogDescription>

        <div role="radiogroup" aria-label={t('nav.country')} className="mt-5 grid gap-2.5">
          {MARKETS.map((code) => {
            const active = selected === code;
            return (
              <button
                key={code}
                type="button"
                role="radio"
                aria-checked={active}
                onClick={() => setSelected(code)}
                className={cn(
                  'flex items-center gap-3 rounded-2xl border px-4 py-3 text-left transition',
                  active ? 'border-primary bg-primary-50 ring-2 ring-primary/30' : 'border-border hover:border-primary-300 hover:bg-primary-50/50',
                )}
              >
                <Flag code={code} className="text-2xl" />
                <span className="flex-1">
                  <span className="block font-semibold">{t(`market.names.${code}`)}</span>
                  <span className="block text-sm text-ink-muted">{MARKET_INFO[code].currency}</span>
                </span>
                {suggested === code && (
                  <span className="rounded-full bg-secondary/20 px-2.5 py-1 text-xs font-semibold text-ink">{t('market.suggested')}</span>
                )}
                {active && <Check className="size-5 text-primary" />}
              </button>
            );
          })}
        </div>

        <Button size="lg" className="mt-5 w-full" onClick={confirm}>
          {t('market.confirm')}
        </Button>
      </DialogContent>
    </Dialog>
  );
}
