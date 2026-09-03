import { useEffect, useMemo, useState } from 'react'

import { Badge, Empty, Select, Spinner } from '@/components/ui'
import { Modal } from '@/components/ui/Modal'
import { bytes, duration } from '@/lib/format'
import { api, errorMessage } from '@/lib/ipc'
import { useApp } from '@/stores/app'
import { useRepeater } from '@/stores/repeater'
import type { Comparison, DiffEntry } from '@/types/analysis'

import css from './repeater.module.css'

function DiffList({ entries, title }: { entries: DiffEntry[]; title: string }) {
  const t = useApp((s) => s.t)
  if (entries.length === 0) {
    return (
      <>
        <p className={css.sectionTitle}>{title}</p>
        <p className={css.compareLabel}>{t('compare.identical')}</p>
      </>
    )
  }

  const kindClass = (kind: DiffEntry['kind']) =>
    kind === 'added' ? css.diffKindAdded : kind === 'removed' ? css.diffKindRemoved : ''

  return (
    <>
      <p className={css.sectionTitle}>
        {title} · {entries.length}
      </p>
      {entries.map((entry, i) => (
        <div
          key={`${entry.path}-${i}`}
          className={`${css.diffRow} ${entry.volatile ? css.diffVolatile : ''}`}
        >
          <span className={`${css.diffKind} ${kindClass(entry.kind)}`}>
            {t(`compare.${entry.kind}`)}
          </span>
          <span className={css.diffPath} title={entry.volatile ? t('compare.volatile') : undefined}>
            {entry.path}
            {entry.volatile ? ' *' : ''}
          </span>
          <span className={css.diffValue}>{entry.left ?? '—'}</span>
          <span className={css.diffValue}>{entry.right ?? '—'}</span>
        </div>
      ))}
    </>
  )
}

export function CompareDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const t = useApp((s) => s.t)
  const notify = useApp((s) => s.notify)
  const draft = useRepeater((s) => s.draft)
  const results = useRepeater((s) => s.results)

  const options = useMemo(() => {
    const list: { id: string; label: string }[] = []
    if (draft?.source_request_id) {
      list.push({ id: `original:${draft.source_request_id}`, label: t('repeater.original') })
    }
    for (const r of results) {
      list.push({ id: `replay:${r.id}`, label: t('repeater.replay', { n: r.index }) })
    }
    return list
  }, [draft, results, t])

  const [left, setLeft] = useState('')
  const [right, setRight] = useState('')
  const [comparison, setComparison] = useState<Comparison | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!open) return
    setLeft((v) => v || options[0]?.id || '')
    setRight((v) => v || options[1]?.id || options[0]?.id || '')
  }, [open, options])

  useEffect(() => {
    if (!open || !left || !right) return
    setLoading(true)
    api
      .compareResponses(left, right)
      .then(setComparison)
      .catch((error) => notify(errorMessage(error), 'error'))
      .finally(() => setLoading(false))
  }, [open, left, right, notify])

  const rows = comparison
    ? [
        {
          label: t('compare.status'),
          left: String(comparison.left.status ?? '—'),
          right: String(comparison.right.status ?? '—'),
        },
        {
          label: t('compare.size'),
          left: bytes(comparison.left.size),
          right: bytes(comparison.right.size),
        },
        {
          label: t('compare.duration'),
          left: duration(comparison.left.duration_ms),
          right: duration(comparison.right.duration_ms),
        },
        {
          label: t('compare.contentType'),
          left: comparison.left.content_type ?? '—',
          right: comparison.right.content_type ?? '—',
        },
      ]
    : []

  return (
    <Modal open={open} title={t('compare.title')} onClose={onClose} wide>
      {options.length < 2 ? (
        <Empty title={t('compare.pick')} />
      ) : (
        <>
          <div className={css.compareGrid}>
            <span className={css.compareLabel} />
            <Select value={left} onChange={(e) => setLeft(e.target.value)}>
              {options.map((o) => (
                <option key={o.id} value={o.id}>
                  {o.label}
                </option>
              ))}
            </Select>
            <Select value={right} onChange={(e) => setRight(e.target.value)}>
              {options.map((o) => (
                <option key={o.id} value={o.id}>
                  {o.label}
                </option>
              ))}
            </Select>

            {rows.map((row) => (
              <div key={row.label} style={{ display: 'contents' }}>
                <span className={css.compareLabel}>{row.label}</span>
                <span className={css.compareValue}>{row.left}</span>
                <span className={css.compareValue}>{row.right}</span>
              </div>
            ))}
          </div>

          {loading ? <Spinner /> : null}

          {comparison ? (
            <>
              <DiffList entries={comparison.header_diff} title={t('compare.headers')} />
              <div style={{ height: 'var(--space-7)' }} />
              {comparison.body_comparable ? (
                <DiffList entries={comparison.body_diff} title={t('compare.body')} />
              ) : (
                <>
                  <p className={css.sectionTitle}>{t('compare.body')}</p>
                  <Badge>{t('compare.notJson')}</Badge>
                </>
              )}
            </>
          ) : null}
        </>
      )}
    </Modal>
  )
}
