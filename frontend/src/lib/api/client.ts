// API client dùng chung. Trình duyệt gọi /api cùng origin;
// phía máy chủ Next (SSR) gọi thẳng backend qua API_INTERNAL_URL.

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public details?: Record<string, string[]> | null,
    public retryAfter?: number,
  ) {
    super(message);
    this.name = 'ApiError';
  }

  /** Thông báo lỗi đầu tiên của một trường (lỗi 422). */
  field(name: string) {
    return this.details?.[name]?.[0];
  }
}

function baseUrl() {
  if (typeof window !== 'undefined') return '';
  return (process.env.API_INTERNAL_URL || 'http://127.0.0.1:8180').replace(/\/$/, '');
}

type Query = Record<string, string | number | boolean | null | undefined>;

export interface RequestOptions extends Omit<RequestInit, 'body'> {
  query?: Query;
  json?: unknown;
  /** Thời gian chờ tối đa (ms). */
  timeoutMs?: number;
}

export async function api<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { query, json, timeoutMs = 20_000, headers, ...init } = options;

  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(query ?? {})) {
    if (value !== null && value !== undefined && value !== '') search.set(key, String(value));
  }
  const qs = search.toString();
  const url = `${baseUrl()}/api/v1${path}${qs ? `?${qs}` : ''}`;

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  if (init.signal) init.signal.addEventListener('abort', () => controller.abort(), { once: true });

  let response: Response;
  try {
    response = await fetch(url, {
      ...init,
      signal: controller.signal,
      headers: {
        Accept: 'application/json',
        ...(json !== undefined ? { 'Content-Type': 'application/json' } : {}),
        ...headers,
      },
      body: json !== undefined ? JSON.stringify(json) : undefined,
    });
  } catch (error) {
    if (init.signal?.aborted) throw error;
    const timedOut = controller.signal.aborted;
    throw new ApiError(
      0,
      timedOut ? 'TIMEOUT' : 'NETWORK_ERROR',
      timedOut ? 'Máy chủ phản hồi quá lâu, vui lòng thử lại.' : 'Không kết nối được máy chủ. Kiểm tra mạng rồi thử lại.',
    );
  } finally {
    clearTimeout(timer);
  }

  if (response.status === 204) return undefined as T;

  let payload: unknown = null;
  try {
    payload = await response.json();
  } catch {
    // Thân phản hồi không phải JSON (ví dụ lỗi 502 từ proxy).
  }

  if (!response.ok) {
    const error = (payload as { error?: { code?: string; message?: string; details?: Record<string, string[]> } } | null)?.error;
    const retryAfter = Number(response.headers.get('retry-after')) || undefined;
    throw new ApiError(
      response.status,
      error?.code ?? 'HTTP_ERROR',
      error?.message ?? 'Hệ thống đang gặp sự cố, vui lòng thử lại sau.',
      error?.details ?? null,
      retryAfter,
    );
  }
  return payload as T;
}

export function errorMessage(error: unknown, fallback = 'Đã có lỗi xảy ra, vui lòng thử lại.') {
  if (error instanceof ApiError) return error.message;
  return fallback;
}
