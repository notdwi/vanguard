import { Badge } from '@/components/ui'
import { CopyButton } from '@/components/ui/CopyButton'
import { sequence } from '@/lib/format'
import { useApp } from '@/stores/app'
import type { RequestDetail } from '@/types/core'

import { maskValue } from './HeaderTable'
import css from './inspector.module.css'

export function CookiesTab({ detail }: { detail: RequestDetail }) {
  const t = useApp((s) => s.t)
  const masked = useApp((s) => s.settings.mask_secrets)
  const { request_cookies: sent, response_cookies: set, cookie_origins: origins } = detail

  if (sent.length === 0 && set.length === 0) {
    return <p className={css.reason}>{t('cookies.none')}</p>
  }

  const show = (value: string) => (masked ? maskValue(value) : value)

  return (
    <div>
      {sent.length > 0 ? (
        <section className={css.section}>
          <div className={css.sectionHead}>
            <span className={css.sectionTitle}>{t('cookies.sent')}</span>
            <div className={css.sectionSpacer} />
            <Badge>{sent.length}</Badge>
          </div>
          <table className={css.table}>
            <tbody>
              {sent.map((c, i) => (
                <tr key={`${c.name}-${i}`}>
                  <td className={css.nameCell}>{c.name}</td>
                  <td className={css.valueCell}>{show(c.value)}</td>
                  <td className={css.rowActions}>
                    <CopyButton value={c.value} size={12} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      ) : null}

      {set.length > 0 ? (
        <section className={css.section}>
          <div className={css.sectionHead}>
            <span className={css.sectionTitle}>{t('cookies.set')}</span>
            <div className={css.sectionSpacer} />
            <Badge>{set.length}</Badge>
          </div>
          <table className={css.table}>
            <tbody>
              {set.map((c, i) => (
                <tr key={`${c.name}-${i}`}>
                  <td className={css.nameCell}>{c.name}</td>
                  <td className={css.valueCell}>
                    {show(c.value)}
                    <div style={{ marginTop: 4, display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                      {c.domain ? <span className={css.pill}>domain={c.domain}</span> : null}
                      <span className={css.pill}>path={c.path}</span>
                      {c.secure ? <span className={css.pill}>secure</span> : null}
                      {c.http_only ? <span className={css.pill}>httponly</span> : null}
                      {c.same_site ? (
                        <span className={css.pill}>samesite={c.same_site}</span>
                      ) : null}
                    </div>
                  </td>
                  <td className={css.rowActions}>
                    <CopyButton value={c.value} size={12} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      ) : null}

      {origins.length > 0 ? (
        <section className={css.section}>
          <div className={css.sectionHead}>
            <span className={css.sectionTitle}>{t('cookies.origin')}</span>
          </div>
          <div className={css.cookieChain}>
            {origins.map((o, i) => (
              <div key={`${o.name}-${i}`} className={css.cookieEvent}>
                <span className={css.seqLink}>{sequence(o.sequence_id)}</span>
                <span className={css.cookieBranch}>
                  {o.direction === 'set' ? 'Set-Cookie' : 'Cookie'}
                </span>
                <span>{o.name}</span>
                <span className={css.cookieBranch}>= {show(o.value_preview)}</span>
              </div>
            ))}
          </div>
        </section>
      ) : null}
    </div>
  )
}
