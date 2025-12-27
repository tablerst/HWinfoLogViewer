export type AppLocale = 'zh-CN' | 'en-US';

export const APP_LOCALE_STORAGE_KEY = 'hwinfo-log-viewer:locale:v1';

export function normalizeLocale(raw: unknown): AppLocale {
  if (raw === 'zh-CN' || raw === 'en-US') return raw;

  // Accept short tags.
  if (raw === 'zh' || raw === 'zh-cn' || raw === 'zh-CN') return 'zh-CN';
  if (raw === 'en' || raw === 'en-us' || raw === 'en-US') return 'en-US';

  // Accept browser style tags (e.g. en-US, zh-CN, zh-Hans-CN)
  if (typeof raw === 'string') {
    const s = raw.trim();
    if (s.toLowerCase().startsWith('zh')) return 'zh-CN';
    if (s.toLowerCase().startsWith('en')) return 'en-US';
  }

  return 'zh-CN';
}

export function loadPersistedLocale(defaultLocale: AppLocale = 'zh-CN'): AppLocale {
  try {
    const saved = localStorage.getItem(APP_LOCALE_STORAGE_KEY);
    if (saved) return normalizeLocale(saved);
  } catch {
    // ignore
  }

  try {
    // If nothing is saved, use browser preference when available.
    if (typeof navigator !== 'undefined' && navigator.language) {
      return normalizeLocale(navigator.language);
    }
  } catch {
    // ignore
  }

  return defaultLocale;
}

export function persistLocale(locale: AppLocale): void {
  try {
    localStorage.setItem(APP_LOCALE_STORAGE_KEY, locale);
  } catch {
    // ignore
  }
}
