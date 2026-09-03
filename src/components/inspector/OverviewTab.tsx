import { Badge, KeyValue } from '@/components/ui'
import { bytes, clockTime, duration, sequence } from '@/lib/format'
import { useApp } from '@/stores/app'
import type { RequestDetail } from '@/types/core'

import css from './inspector.module.css'

export function OverviewTab({ detail }: { detail: RequestDetail }) {
  const t = useApp((s) => s.t)
  const r = detail.request
  const res = r.response

  const items = [
    { key: t('field.sequence'), value: sequence(r.sequence_id) },
    { key: t('field.method'), value: r.method },
    { key: t('field.url'), value: r.url },
    { key: t('field.endpoint'), value: r.normalized_path },
    { key: t('field.protocol'), value: r.protocol },
    {
      key: t('field.status'),
      value: res ? `${res.status} ${res.status_text}` : r.error ? 'failed' : '—',
    },
    { key: t('field.started'), value: clockTime(r.timestamp) },
    { key: t('field.duration'), value: res ? duration(res.duration_ms) : '—' },
    { key: t('field.requestSize'), value: bytes(r.request_size) },
    { key: t('field.responseSize'), value: res ? bytes(res.body.size) : '—' },
    {
      key: t('field.contentType'),
      value: res?.content_type ?? r.request_content_type ?? '—',
    },
    { key: t('field.clientAddr'), value: r.client_addr ?? '—' },
    { key: t('field.remoteIp'), value: r.remote_ip ?? '—' },
  ]

  return (
    <div>
      <div className={css.section}>
        <div className={css.sectionHead}>
          <span className={css.sectionTitle}>{t('tab.overview')}</span>
          <div className={css.sectionSpacer} />
          <Badge tone={r.importance === 'high' ? 'solid' : 'default'}>
            {t(`importance.${r.importance}`)}
          </Badge>
        </div>
        <KeyValue items={items} />
      </div>

      {r.error ? (
        <div className={css.section}>
          <div className={css.sectionHead}>
            <span className={css.sectionTitle}>{t('common.error')}</span>
          </div>
          <p className={css.valueCell}>{r.error}</p>
        </div>
      ) : null}

      {r.importance_reasons.length > 0 ? (
        <div className={css.section}>
          <div className={css.sectionHead}>
            <span className={css.sectionTitle}>{t('analysis.whyImportant')}</span>
          </div>
          <ul className={css.reasonList}>
            {r.importance_reasons.map((reason) => (
              <li key={reason} className={css.reason}>
                {reason}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  )
}
