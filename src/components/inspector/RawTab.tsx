import { useEffect, useState } from 'react'

import { CopyButton } from '@/components/ui/CopyButton'
import { api } from '@/lib/ipc'
import { useApp } from '@/stores/app'
import type { RequestDetail } from '@/types/core'

import viewers from '@/components/viewers/viewers.module.css'
import css from './inspector.module.css'

function buildRequest(detail: RequestDetail, body: string): string {
  const r = detail.request
  const target = `${r.path}${r.query ? `?${r.query}` : ''}`
  const lines = [`${r.method} ${target} ${r.protocol}`]
  for (const h of r.request_headers) lines.push(`${h.name}: ${h.value}`)
  return `${lines.join('\n')}\n\n${body}`
}

function buildResponse(detail: RequestDetail, body: string): string {
  const res = detail.request.response
  if (!res) return ''
  const lines = [`${res.protocol} ${res.status} ${res.status_text}`]
  for (const h of res.headers) lines.push(`${h.name}: ${h.value}`)
  return `${lines.join('\n')}\n\n${body}`
}

export function RawTab({ detail }: { detail: RequestDetail }) {
  const t = useApp((s) => s.t)
  const [side, setSide] = useState<'request' | 'response'>('request')
  const [body, setBody] = useState('')

  useEffect(() => {
    let cancelled = false
    setBody('')
    const ref =
      side === 'request' ? detail.request.request_body : detail.request.response?.body
    if (!ref || ref.size === 0 || !ref.is_text || ref.storage === 'skipped') return
    void api
      .loadBody(detail.request.id, side, false)
      .then((payload) => {
        if (!cancelled) setBody(payload.content ?? '')
      })
      .catch(() => undefined)
    return () => {
      cancelled = true
    }
  }, [detail, side])

  const text =
    side === 'request' ? buildRequest(detail, body) : buildResponse(detail, body)

  return (
    <div className={viewers.bodyWrap}>
      <div className={viewers.bodyBar}>
        <div className={viewers.segmented}>
          <button
            type="button"
            className={`${viewers.segment} ${side === 'request' ? viewers.segmentActive : ''}`}
            onClick={() => setSide('request')}
          >
            {t('headers.request')}
          </button>
          <button
            type="button"
            className={`${viewers.segment} ${side === 'response' ? viewers.segmentActive : ''}`}
            onClick={() => setSide('response')}
            disabled={!detail.request.response}
          >
            {t('headers.response')}
          </button>
        </div>
        <div className={viewers.bodySpacer} />
        <CopyButton value={text} />
      </div>
      <div className={viewers.bodyContent}>
        {text ? (
          <pre className={viewers.pre}>{text}</pre>
        ) : (
          <p className={css.reason} style={{ padding: 16 }}>
            {t('body.empty')}
          </p>
        )}
      </div>
    </div>
  )
}
