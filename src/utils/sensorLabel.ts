export type SensorLabelMeta = {
  raw: string;
  baseName: string;
  unit: string | null;
};

// Parse a sensor field label like "下载总计 [MB]".
// - Unit is taken from the LAST trailing "[ ... ]".
// - Empty unit "[]" is treated as no unit.
export function parseSensorLabel(raw: string): SensorLabelMeta {
  const s = (raw ?? '').toString();
  const m = s.match(/^(.*?)(?:\s*\[([^\]]*)\])\s*$/);
  if (!m) {
    return { raw: s, baseName: s.trim(), unit: null };
  }

  const baseName = (m[1] ?? '').trim();
  const unitText = (m[2] ?? '').trim();
  const unit = unitText.length ? unitText : null;
  return {
    raw: s,
    baseName: baseName.length ? baseName : s.trim(),
    unit,
  };
}

type UnitFormatRule = {
  maximumFractionDigits: number;
  minimumFractionDigits?: number;
  useGrouping?: boolean;
};

function pickRule(unitRaw: string | null, value: number): UnitFormatRule {
  const unit = (unitRaw ?? '').trim();
  const abs = Math.abs(value);

  // Common units
  if (unit === '%') {
    return { maximumFractionDigits: 1, minimumFractionDigits: 0, useGrouping: false };
  }
  if (unit === '℃' || unit === '°C' || unit === '°F') {
    return { maximumFractionDigits: 1, minimumFractionDigits: 0, useGrouping: false };
  }
  if (unit === 'V') {
    return { maximumFractionDigits: 3, minimumFractionDigits: 0, useGrouping: false };
  }
  if (unit === 'A') {
    return { maximumFractionDigits: 3, minimumFractionDigits: 0, useGrouping: false };
  }
  if (unit === 'W') {
    return { maximumFractionDigits: 1, minimumFractionDigits: 0, useGrouping: true };
  }
  if (unit === 'RPM') {
    return { maximumFractionDigits: 0, minimumFractionDigits: 0, useGrouping: true };
  }
  if (unit === 'MHz' || unit === 'kHz') {
    return { maximumFractionDigits: 0, minimumFractionDigits: 0, useGrouping: true };
  }
  if (unit === 'GHz') {
    return { maximumFractionDigits: 1, minimumFractionDigits: 0, useGrouping: true };
  }

  // Bytes / storage
  if (/^(B|KB|KiB|MB|MiB|GB|GiB|TB|TiB)$/i.test(unit)) {
    if (/^(GB|GiB)$/i.test(unit)) return { maximumFractionDigits: 1, minimumFractionDigits: 0, useGrouping: true };
    return { maximumFractionDigits: 0, minimumFractionDigits: 0, useGrouping: true };
  }

  // Rates (e.g. MB/s, KB/s)
  if (/\/(s|sec)$/i.test(unit)) {
    return { maximumFractionDigits: 1, minimumFractionDigits: 0, useGrouping: true };
  }

  // Default heuristic: keep it readable without over-precision.
  if (abs === 0) return { maximumFractionDigits: 0, minimumFractionDigits: 0, useGrouping: true };
  if (abs < 10) return { maximumFractionDigits: 2, minimumFractionDigits: 0, useGrouping: true };
  if (abs < 100) return { maximumFractionDigits: 1, minimumFractionDigits: 0, useGrouping: true };
  return { maximumFractionDigits: 0, minimumFractionDigits: 0, useGrouping: true };
}

export function formatValueByUnit(
  value: unknown,
  unit: string | null,
  opts?: { locale?: string }
): string {
  const n = typeof value === 'number' ? value : Number(value);
  if (!Number.isFinite(n)) return '-';

  const rule = pickRule(unit, n);
  const locale = opts?.locale ?? 'zh-CN';

  try {
    return new Intl.NumberFormat(locale, {
      useGrouping: rule.useGrouping ?? true,
      maximumFractionDigits: rule.maximumFractionDigits,
      minimumFractionDigits: rule.minimumFractionDigits ?? 0,
    }).format(n);
  } catch {
    // Very old runtime fallback
    return String(n);
  }
}

export function formatValueWithUnit(value: unknown, unit: string | null, opts?: { locale?: string }): string {
  const valText = formatValueByUnit(value, unit, opts);
  if (valText === '-') return '-';

  const u = (unit ?? '').trim();
  if (!u) return valText;

  // No-space suffix for some symbols.
  if (u === '%' || u.startsWith('°') || u === '℃') return `${valText}${u}`;
  return `${valText} ${u}`;
}
