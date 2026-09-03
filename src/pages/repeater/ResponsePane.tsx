import { useState } from 'react'

import { Badge, Empty, Tabs } from '@/components/ui'
import { CopyButton } from '@/components/ui/CopyButton'
import { HeaderTable } from '@/components/inspector/HeaderTable'
import { JsonViewer } from '@/components/viewers/JsonViewer'
import viewers from '@/components/viewers/viewers.module.css'
import { bytes, duration, isJsonText, statusClass } from '@/lib/format'
import { useApp } from '@/stores/app'
import type { ReplayResult } from '@/types/repeater'

import css from './repeater.module.css'

export function ResponsePane({ result }: { result: ReplayResult | null }) {
  const t = useApp((s) => s.t)
  const masked = useApp((s) => s.settings.mask_secrets)
  const [tab, setTab] = useState<'body' | 'headers' | 'sent'>('body')

  if (!result) {
    return <Empty title={t('repeater.noResponse')} />
  }

  const json = result.body_is_text && isJsonText(result.body)

  return (
    <>
      <div className={css.responseHead}>
        <span className={`${css.statusText} ${statusClass(result.status)}`}>
          {result.status ?? 'ERR'} {result.status_text}
        </span>
        <span className={css.responseMeta}>{duration(result.duration_ms)}</span>
        <span className={css.responseMeta}>{bytes(result.body_size)}</span>
        {result.content_type ? <Badge>{result.content_type.split(';')[0]}</Badge> : null}
        <div className={css.spacer} />
        <CopyButton value={result.body ?? ''} label={t('action.copyBody')} />
      </div>

      <Tabs
        items={[
          { id: 'body', label: t('tab.body') },
          { id: 'headers', label: t('tab.headers'), count: result.headers.length },
          { id: 'sent', label: t('tab.raw') },
        ]}
        active={tab}
        onSelect={(id) => setTab(id as typeof tab)}
      />

      <div className={css.paneFlush}>
        {result.error ? (
          <div className={viewers.notice}>{result.error}</div>
        ) : tab === 'body' ? (
          json ? (
            <JsonViewer text={result.body ?? ''} />
          ) : (
            <div className={viewers.bodyContent}>
              <pre className={`${viewers.pre} ${viewers.preWrap}`}>
                {result.body_is_text ? result.body : t('body.binary')}
              </pre>
            </div>
          )
        ) : tab === 'headers' ? (
          <div className={css.paneBody}>
            <HeaderTable headers={result.headers} masked={masked} />
          </div>
        ) : (
          <div className={viewers.bodyContent}>
            <pre className={viewers.pre}>
              {[
                `${result.sent.method} ${result.sent.url}`,
                ...result.sent.headers.map((h) => `${h.name}: ${h.value}`),
                '',
                result.sent.body,
              ].join('\n')}
            </pre>
          </div>
        )}
      </div>
    </>
  )
}
