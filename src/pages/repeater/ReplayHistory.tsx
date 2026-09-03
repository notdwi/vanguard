import { Badge, Button, Empty } from '@/components/ui'
import { bytes, clockTime, duration, statusClass } from '@/lib/format'
import { useApp } from '@/stores/app'
import { useRepeater } from '@/stores/repeater'
import type { ReplayResult } from '@/types/repeater'

import css from './repeater.module.css'

export function ReplayHistory({
  selectedId,
  onSelect,
  onCompare,
}: {
  selectedId: string | null
  onSelect: (result: ReplayResult) => void
  onCompare: () => void
}) {
  const t = useApp((s) => s.t)
  const results = useRepeater((s) => s.results)
  const clearHistory = useRepeater((s) => s.clearHistory)

  return (
    <>
      <div className={css.sectionHead} style={{ padding: 'var(--space-4) var(--space-5) 0' }}>
        <span className={css.sectionTitle}>{t('repeater.history')}</span>
        <Badge>{results.length}</Badge>
        <div className={css.spacer} />
        <Button small icon="analysis" onClick={onCompare} disabled={results.length === 0}>
          {t('compare.open')}
        </Button>
        <Button small icon="trash" onClick={() => void clearHistory()} disabled={results.length === 0}>
          {t('repeater.clearHistory')}
        </Button>
      </div>

      {results.length === 0 ? (
        <Empty title={t('repeater.noHistory')} />
      ) : (
        <div className={css.historyList}>
          {results.map((result) => (
            <button
              key={result.id}
              type="button"
              className={`${css.historyRow} ${selectedId === result.id ? css.historyRowActive : ''}`}
              onClick={() => onSelect(result)}
            >
              <span className={css.historyIndex}>#{result.index}</span>
              <span className={result.error ? css.historyError : statusClass(result.status)}>
                {result.status ?? 'ERR'}
              </span>
              <span className={css.historyIndex}>{duration(result.duration_ms)}</span>
              <span className={css.historyIndex}>{bytes(result.body_size)}</span>
              <span className={css.historyIndex}>{clockTime(result.started_at)}</span>
            </button>
          ))}
        </div>
      )}
    </>
  )
}
