import type { Locale } from './config';
import type { Messages } from './messages/vi';

export type { Messages };

/** Tải từ điển theo ngôn ngữ (tách chunk theo ngôn ngữ, chỉ tải bản đang dùng). */
export async function loadMessages(locale: Locale): Promise<Messages> {
  switch (locale) {
    case 'en':
      return (await import('./messages/en')).default;
    case 'ja':
      return (await import('./messages/ja')).default;
    case 'ko':
      return (await import('./messages/ko')).default;
    case 'zh':
      return (await import('./messages/zh')).default;
    default:
      return (await import('./messages/vi')).default;
  }
}

type Vars = Record<string, string | number>;

/** Lấy chuỗi theo đường dẫn "a.b.c" và thay {biến}. Thiếu khoá thì trả chính đường dẫn để dễ phát hiện. */
export function translate(messages: Messages, key: string, vars?: Vars): string {
  let node: unknown = messages;
  for (const part of key.split('.')) {
    node = node && typeof node === 'object' ? (node as Record<string, unknown>)[part] : undefined;
  }
  if (typeof node !== 'string') return key;
  if (!vars) return node;
  return node.replace(/\{(\w+)\}/g, (match, name: string) => (name in vars ? String(vars[name]) : match));
}
