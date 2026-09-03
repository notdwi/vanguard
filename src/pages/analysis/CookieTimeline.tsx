import { Badge, Empty } from '@/components/ui'
import { sequence } from '@/lib/format'
import { useApp } from '@/stores/app'
import type { CookieEvent, CookieUsage } from '@/types/analysis'

import css from './analysis.module.css'

function Chain({
  label,
  events,
  onOpen,
}: {
  label: string
  events: CookieEvent[]
  onOpen: (requestId: string) => void
}) {
  if (events.length === 0) return null
  return (
    <div className={css.chainGroup}>
      <div className={css.chainLabel}>
        {label} · {events.length}
      </div>
      {events.slice(0, 25).map((e, i) => (
        <div key={`${e.request_id}-${i}`} className={css.chainRow}>
          <button type="button" className={css.seqLink} onClick={() => onOpen(e.request_id)}>
            {sequence(e.sequence_id)}
          </button>
          <span className={css.chainMethod}>{e.method}</span>
          <span>{e.path}</span>
        </div>
      ))}
      {events.length > 25 ? (
        <div className={css.chainRow}>+{events.length - 25}</div>
      ) : null}
    </div>
  )
}

export function CookieTimeline({
  cookies,
  onOpen,
}: {
  cookies: CookieUsage[]
  onOpen: (requestId: string) => void
}) {
  const t = useApp((s) => s.t)

  if (cookies.length === 0) {
    return <Empty title={t('cookies.none')} />
  }

  return (
    <div>
      {cookies.map((cookie) => (
        <div key={`${cookie.name}-${cookie.domain}`} className={css.cookieCard}>
          <div className={css.cookieHead}>
            <span className={css.cookieName}>{cookie.name}</span>
            <span className={css.cookieDomain}>{cookie.domain}</span>
            <div style={{ flex: 1 }} />
            <Badge>{t('cookies.distinctValues', { n: cookie.distinct_values })}</Badge>
          </div>
          <div className={css.cookieBody}>
            <Chain label={t('cookies.createdBy')} events={cookie.created_by} onOpen={onOpen} />
            <Chain label={t('cookies.usedBy')} events={cookie.used_by} onOpen={onOpen} />
          </div>
        </div>
      ))}
    </div>
  )
}
