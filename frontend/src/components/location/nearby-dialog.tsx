'use client';

import { MapPinned, Pencil } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { useI18n } from '@/i18n/provider';
import { useLocationStore } from '@/stores/location-store';
import { useNearbyDialog } from '@/stores/nearby-dialog-store';

import { LocationPicker } from './location-picker';
import { NearbyPlaces } from './nearby-places';

/**
 * Hộp thoại "Tìm quán gần nhất" dùng chung toàn trang.
 * Bước 1: xin phép dùng vị trí (hoặc nhập địa chỉ). Bước 2: danh sách quán gần nhất và nút mở Google Maps chỉ đường.
 */
export function NearbyDialog() {
  const { t } = useI18n();
  const { target, close } = useNearbyDialog();
  const location = useLocationStore((s) => s.location);
  const [changing, setChanging] = useState(false);

  useEffect(() => {
    if (!target) setChanging(false);
  }, [target]);

  const askLocation = !location || changing;

  return (
    <Dialog open={!!target} onOpenChange={(open) => !open && close()}>
      <DialogContent className="max-w-xl">
        {target && (
          <>
            <div className="flex items-start gap-3">
              <div className="grid size-12 shrink-0 place-items-center rounded-full bg-primary-50 text-primary">
                <MapPinned className="size-6" />
              </div>
              <div>
                <DialogTitle>{askLocation ? t('nearby.askTitle') : t('nearby.title')}</DialogTitle>
                <DialogDescription className="mt-1">
                  {askLocation ? t('nearby.askDesc', { name: target.name }) : t('nearby.desc', { name: target.name })}
                </DialogDescription>
              </div>
            </div>

            <div className="mt-5">
              {askLocation ? (
                <LocationPicker onPicked={() => setChanging(false)} />
              ) : (
                <>
                  <div className="mb-3 flex items-center justify-between gap-3 rounded-2xl bg-background px-4 py-2.5 text-sm">
                    <span className="min-w-0 truncate">
                      <span className="text-ink-muted">{t('nearby.location')} </span>
                      <span className="font-semibold">{location.label}</span>
                    </span>
                    <button
                      type="button"
                      onClick={() => setChanging(true)}
                      className="flex shrink-0 items-center gap-1 font-semibold text-primary-700 hover:underline"
                    >
                      <Pencil className="size-3.5" /> {t('nearby.change')}
                    </button>
                  </div>
                  <NearbyPlaces target={target} />
                </>
              )}
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
