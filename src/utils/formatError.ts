export function formatError(err: unknown): string {
  if (err == null) return '未知错误'

  // Tauri invoke 失败时可能直接抛出字符串
  if (typeof err === 'string') return err

  // 常规 Error
  if (err instanceof Error) {
    // 部分运行时会把更多信息挂在 cause
    const cause = (err as { cause?: unknown }).cause
    if (cause != null) return `${err.message}\n${formatError(cause)}`
    return err.message || '未知错误'
  }

  // 兜底：尽量 stringify
  try {
    return JSON.stringify(err)
  } catch {
    return String(err)
  }
}
