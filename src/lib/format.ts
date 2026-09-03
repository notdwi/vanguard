export function bytes(value: number): string {
  if (!value) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  let n = value
  let i = 0
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024
    i += 1
  }
  return `${i === 0 ? n : n.toFixed(n < 10 ? 1 : 0)} ${units[i]}`
}

export function duration(ms: number | null | undefined): string {
  if (ms == null) return '—'
  if (ms < 1000) return `${ms}ms`
  if (ms < 60_000) return `${(ms / 1000).toFixed(2)}s`
  return `${Math.floor(ms / 60_000)}m ${Math.round((ms % 60_000) / 1000)}s`
}

export function clockTime(ms: number): string {
  const d = new Date(ms)
  const pad = (n: number, w = 2) => String(n).padStart(w, '0')
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}.${pad(
    d.getMilliseconds(),
    3,
  )}`
}

export function dateLabel(ms: number): string {
  return new Date(ms).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
  })
}

export function relativeDay(ms: number): string {
  const now = new Date()
  const then = new Date(ms)
  const startOf = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
  const days = Math.round((startOf(now) - startOf(then)) / 86_400_000)
  if (days === 0) return 'Today'
  if (days === 1) return 'Yesterday'
  if (days < 7) return `${days} days ago`
  return dateLabel(ms)
}

export function sequence(id: number): string {
  return `#${String(id).padStart(3, '0')}`
}

export function statusClass(status: number | null | undefined): string {
  if (status == null) return 'pending'
  if (status < 200) return 'info'
  if (status < 300) return 'ok'
  if (status < 400) return 'redirect'
  if (status < 500) return 'client'
  return 'server'
}

export function shortHost(host: string): string {
  const parts = host.split('.')
  if (parts.length <= 2) return host
  return parts.slice(0, -2).join('.') || host
}

export function prettyJson(text: string): string {
  try {
    return JSON.stringify(JSON.parse(text), null, 2)
  } catch {
    return text
  }
}

export function isJsonText(text: string | null | undefined): boolean {
  if (!text) return false
  const t = text.trimStart()
  if (!t.startsWith('{') && !t.startsWith('[')) return false
  try {
    JSON.parse(text)
    return true
  } catch {
    return false
  }
}

export function count(n: number): string {
  return n.toLocaleString()
}
