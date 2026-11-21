'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { ApiError, authApi } from '@/lib/api';
import type { UserProfile } from '@/lib/types';

export const ME_KEY = ['auth', 'me'] as const;

/** Tài khoản đang đăng nhập; `null` nếu là khách. */
export function useMe() {
  return useQuery<UserProfile | null>({
    queryKey: ME_KEY,
    queryFn: async () => {
      try {
        return await authApi.me();
      } catch (error) {
        if (error instanceof ApiError && error.status === 401) return null;
        throw error;
      }
    },
    staleTime: 5 * 60_000,
    retry: false,
  });
}

export function useLogout() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: authApi.logout,
    onSuccess: () => {
      queryClient.setQueryData(ME_KEY, null);
      queryClient.removeQueries({ queryKey: ['favorites'] });
    },
  });
}
