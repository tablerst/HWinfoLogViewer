export function parseHwinfoDateTimeToMs(dateStr: unknown, timeStr: unknown): number | null {
  if (typeof dateStr !== 'string' || typeof timeStr !== 'string') return null;

  const date = dateStr.trim();
  const time = timeStr.trim();
  if (!date || !time) return null;

  // Date examples seen in sample CSV: 22.3.2025
  // Also support: 2025-03-22, 22/3/2025, 3/22/2025
  const dateParts = date.split(/[./-]/).map(p => p.trim()).filter(Boolean);
  if (dateParts.length !== 3) return null;

  const yearIndex = dateParts.findIndex(p => /^\d{4}$/.test(p));
  if (yearIndex === -1) return null;

  const year = Number(dateParts[yearIndex]);
  if (!Number.isFinite(year) || year < 1970 || year > 3000) return null;

  let month: number;
  let day: number;

  if (yearIndex === 0) {
    // YYYY-M-D
    month = Number(dateParts[1]);
    day = Number(dateParts[2]);
  } else if (yearIndex === 2) {
    // D.M.YYYY or M.D.YYYY
    const a = Number(dateParts[0]);
    const b = Number(dateParts[1]);
    if (!Number.isFinite(a) || !Number.isFinite(b)) return null;

    // Heuristic:
    // - If first part > 12 => assume D.M.YYYY
    // - Else if second part > 12 => assume M.D.YYYY
    // - Else default to D.M.YYYY (fits many non-US locales and sample CSV)
    if (a > 12) {
      day = a;
      month = b;
    } else if (b > 12) {
      month = a;
      day = b;
    } else {
      day = a;
      month = b;
    }
  } else {
    // Rare/unsupported ordering
    return null;
  }

  if (!Number.isFinite(month) || !Number.isFinite(day)) return null;
  if (month < 1 || month > 12) return null;
  if (day < 1 || day > 31) return null;

  // Time examples: 21:36:49.335
  const m = time.match(/^(\d{1,2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?$/);
  if (!m) return null;

  const hh = Number(m[1]);
  const mm = Number(m[2]);
  const ss = Number(m[3]);
  const msRaw = m[4];
  const ms = msRaw ? Number(msRaw.padEnd(3, '0')) : 0;

  if ([hh, mm, ss, ms].some(n => !Number.isFinite(n))) return null;
  if (hh < 0 || hh > 23) return null;
  if (mm < 0 || mm > 59) return null;
  if (ss < 0 || ss > 59) return null;
  if (ms < 0 || ms > 999) return null;

  const d = new Date(year, month - 1, day, hh, mm, ss, ms);
  const ts = d.getTime();
  return Number.isFinite(ts) ? ts : null;
}

export function formatDateTimeForTooltip(ms: number): string {
  const d = new Date(ms);
  const yyyy = d.getFullYear();
  const MM = String(d.getMonth() + 1).padStart(2, '0');
  const dd = String(d.getDate()).padStart(2, '0');
  const hh = String(d.getHours()).padStart(2, '0');
  const mm = String(d.getMinutes()).padStart(2, '0');
  const ss = String(d.getSeconds()).padStart(2, '0');
  const mss = String(d.getMilliseconds()).padStart(3, '0');
  return `${yyyy}-${MM}-${dd} ${hh}:${mm}:${ss}.${mss}`;
}

export function formatTimeTick(ms: number, spanMs: number): string {
  const d = new Date(ms);
  const hh = String(d.getHours()).padStart(2, '0');
  const mm = String(d.getMinutes()).padStart(2, '0');

  // If spanning multiple days, include date for readability.
  if (spanMs >= 36 * 60 * 60 * 1000) {
    const MM = String(d.getMonth() + 1).padStart(2, '0');
    const dd = String(d.getDate()).padStart(2, '0');
    return `${MM}-${dd} ${hh}:${mm}`;
  }

  return `${hh}:${mm}`;
}
