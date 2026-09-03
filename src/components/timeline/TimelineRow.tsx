import { memo } from 'react'

import { bytes, duration, sequence, statusClass } from '@/lib/format'
import type { TimelineRow as Row } from '@/types/core'

import css from './timeline.module.css'

const WRITE_METHODS = new Set(['POST', 'PUT', 'PATCH', 'DELETE'])

interface Props {
  row: Row
  selected: boolean
  offset: number
  onSelect: (id: string) => void
  onContextMenu: (event: React.MouseEvent, row: Row) => void
}

function statusStyle(status: number | null): string {
  const kind = statusClass(status)
  const map: Record<string, string> = {
    ok: css.statusOk,
    redirect: css.statusRedirect,
    client: css.statusClient,
    server: css.statusServer,
    pending: css.statusPending,
    info: css.statusOk,
  }
  return map[kind] ?? css.statusPending
}

function TimelineRowBase({ row, selected, offset, onSelect, onContextMenu }: Props) {
  const markClass =
    row.importance === 'high'
      ? css.markHigh
      : row.importance === 'medium'
        ? css.markMedium
        : css.markLow

  return (
    <div
      className={`${css.row} ${selected ? css.rowSelected : ''}`}
      style={{ transform: `translateY(${offset}px)` }}
      onClick={() => onSelect(row.id)}
      onContextMenu={(e) => onContextMenu(e, row)}
      role="row"
      aria-selected={selected}
      tabIndex={-1}
    >
      <span className={css.seq}>
        <span className={`${css.mark} ${markClass}`} /> {sequence(row.sequence_id)}
      </span>
      <span className={`${css.method} ${WRITE_METHODS.has(row.method) ? css.methodWrite : ''}`}>
        {row.method}
      </span>
      <span className={`${css.status} ${statusStyle(row.status)}`}>
        <span className={css.statusBar} />
        {row.has_error ? <span className={css.errorFlag}>ERR</span> : (row.status ?? '···')}
      </span>
      <span className={css.host} title={row.host}>
        {row.host}
      </span>
      <span className={css.path}>
        <span className={css.pathText} title={row.path}>
          {row.path}
        </span>
        {row.query ? <span className={css.query}>?{row.query}</span> : null}
      </span>
      <span className={css.type}>{row.family ?? ''}</span>
      <span className={css.numeric}>{row.response_size ? bytes(row.response_size) : ''}</span>
      <span className={css.numeric}>{row.duration_ms != null ? duration(row.duration_ms) : ''}</span>
    </div>
  )
}

export const TimelineRow = memo(TimelineRowBase)
