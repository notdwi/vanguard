import { useEffect, useState } from 'react'

import { Button, Spinner } from '@/components/ui'
import { CopyButton } from '@/components/ui/CopyButton'
import { bytes, isJsonText, prettyJson } from '@/lib/format'
import { api, errorMessage } from '@/lib/ipc'
import { useApp } from '@/stores/app'
import type { BodyPayload, BodyRef } from '@/types/core'

import { JsonViewer } from './JsonViewer'
import css from './viewers.module.css'

type Mode = 'pretty' | 'raw'

const AUTO_LOAD_LIMIT = 512 * 1024

export function BodyViewer({
  requestId,
  side,
  reference,
  maxBodyBytes,
}: {
  requestId: string
  side: 'request' | 'response'
  reference: BodyRef
  maxBodyBytes: number
}) {
  const t = useApp((s) => s.t)
  const notify = useApp((s) => s.notify)
  const [payload, setPayload] = useState<BodyPayload | null>(null)
  const [loading, setLoading] = useState(false)
  const [mode, setMode] = useState<Mode>('pretty')
  const [wrap, setWrap] = useState(false)

  const heavy = reference.size > AUTO_LOAD_LIMIT

  const fetchBody = async (full: boolean) => {
    setLoading(true)
    try {
      setPayload(await api.loadBody(requestId, side, full))
    } catch (error) {
      notify(errorMessage(error), 'error')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    setPayload(null)
    setMode('pretty')
    if (reference.storage === 'none' || reference.storage === 'skipped') return
    if (heavy) return
    void fetchBody(false)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [requestId, side, reference.storage, reference.size])

  if (reference.storage === 'none' || reference.size === 0) {
    return <div className={css.notice}>{t('body.empty')}</div>
  }

  if (reference.storage === 'skipped') {
    return (
      <div className={css.notice}>
        <strong>{t('body.skipped')}</strong>
        <span>{t('body.skippedHint', { limit: bytes(maxBodyBytes) })}</span>
        <span className={css.bodyMeta}>{bytes(reference.size)}</span>
      </div>
    )
  }

  if (!payload) {
    return (
      <div className={css.notice}>
        <span>{t('body.large', { size: bytes(reference.size) })}</span>
        <Button icon="download" onClick={() => void fetchBody(true)} loading={loading}>
          {t('body.load')}
        </Button>
      </div>
    )
  }

  if (loading) {
    return (
      <div className={css.notice}>
        <Spinner />
      </div>
    )
  }

  const content = payload.content ?? ''

  if (!payload.is_text) {
    return (
      <div className={css.notice}>
        <strong>{t('body.binary')}</strong>
        <span className={css.bodyMeta}>
          {bytes(payload.size)} · {payload.kind}
        </span>
        <CopyButton value={content} />
      </div>
    )
  }

  const json = isJsonText(content)
  const shown = mode === 'pretty' && json ? prettyJson(content) : content

  return (
    <div className={css.bodyWrap}>
      <div className={css.bodyBar}>
        {json ? (
          <div className={css.segmented}>
            <button
              type="button"
              className={`${css.segment} ${mode === 'pretty' ? css.segmentActive : ''}`}
              onClick={() => setMode('pretty')}
            >
              {t('body.pretty')}
            </button>
            <button
              type="button"
              className={`${css.segment} ${mode === 'raw' ? css.segmentActive : ''}`}
              onClick={() => setMode('raw')}
            >
              {t('body.raw')}
            </button>
          </div>
        ) : null}
        <span className={css.bodyMeta}>
          {bytes(payload.size)} · {payload.kind}
        </span>
        {payload.truncated ? <span className={css.bodyMeta}>{t('body.truncated')}</span> : null}
        <div className={css.bodySpacer} />
        {!json || mode === 'raw' ? (
          <button
            type="button"
            className={`${css.segment} ${wrap ? css.segmentActive : ''}`}
            onClick={() => setWrap((v) => !v)}
          >
            {t('body.wrap')}
          </button>
        ) : null}
        {payload.truncated ? (
          <Button small icon="download" onClick={() => void fetchBody(true)}>
            {t('body.load')}
          </Button>
        ) : null}
        <CopyButton value={content} />
      </div>
      <div className={css.bodyContent}>
        {json && mode === 'pretty' ? (
          <JsonViewer text={content} />
        ) : (
          <pre className={`${css.pre} ${wrap ? css.preWrap : ''}`}>{shown}</pre>
        )}
      </div>
    </div>
  )
}
