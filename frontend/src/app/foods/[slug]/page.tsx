import type { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { cache } from 'react';

import { FoodDetailView } from '@/components/food/food-detail';
import type { Locale, MarketCode } from '@/i18n/config';
import { getRequestPrefs } from '@/i18n/server';
import { ApiError, foodsApi } from '@/lib/api';

// Nội dung món phụ thuộc ngôn ngữ/thị trường (cookie) nên dựng theo request; dữ liệu fetch được cache 5 phút
// ở tầng fetch của Next và ở Redis phía backend, nên tải cao cũng không dồn về PostgreSQL.
const getFood = cache(async (slug: string, lang: Locale, market: MarketCode) => {
  try {
    return await foodsApi.detail(slug, { lang, market }, { next: { revalidate: 300 } } as RequestInit);
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) return null;
    throw error;
  }
});

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }): Promise<Metadata> {
  const { locale, market } = await getRequestPrefs();
  const food = await getFood((await params).slug, locale, market);
  if (!food) return { title: '404' };
  return {
    title: food.name,
    description: food.description,
    openGraph: { title: food.name, description: food.description, images: [food.image_url] },
  };
}

export default async function FoodPage({ params }: { params: Promise<{ slug: string }> }) {
  const { locale, market } = await getRequestPrefs();
  const food = await getFood((await params).slug, locale, market);
  if (!food) notFound();
  return <FoodDetailView food={food} />;
}
