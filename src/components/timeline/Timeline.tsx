import { useEffect, useMemo, useRef, useState } from 'react'

import { useVirtualizer } from '@tanstack/react-virtual'

import { Badge, Empty, Icon, IconButton, Spinner, Toggle } from '@/components/ui'
import { count } from '@/lib/format'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import { useTimeline } from '@/stores/timeline'
import type { TimelineRow as Row } from '@/types/core'

import { ContextMenu, type MenuPosition } from './ContextMenu'
import { FilterBar } from './FilterBar'
import { TimelineRow } from './TimelineRow'
import css from './timeline.module.css'

const COLUMNS = '78px 52px 62px 150px minmax(160px, 1fr) 62px 66px 62px'
const ROW_HEIGHT = 26

export function Timeline() {
  const t = useApp((s) => s.t)
  const session = useCapture((s) => s.session)
  const status = useCapture((s) => s.status)
  const {
    rows,
    total,
    filters,
    selectedId,
    loading,
    live,
    load,
    select,
    setFilter,
    setLive,
    clearFilters,
    activeFilterCount,
  } = useTimeline()

  const [showFilters, setShowFilters] = useState(false)
  const [menu, setMenu] = useState<{ row: Row; position: MenuPosition } | null>(null)
  const scrollRef = useRef<HTMLDivElement>(null)
  const stickToBottom = useRef(true)

  const activeCount = activeFilterCount()

  useEffect(() => {
    if (!session) return
    void load(session.id)
  }, [session, load, filters])

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 18,
  })

  useEffect(() => {
    if (!live || !stickToBottom.current || rows.length === 0) return
    virtualizer.scrollToIndex(rows.length - 1, { align: 'end' })
  }, [rows.length, live, virtualizer])

  const onScroll = () => {
    const el = scrollRef.current
    if (!el) return
    stickToBottom.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40
  }

  const items = virtualizer.getVirtualItems()
  const capturing = status.state === 'capturing'

  const body = useMemo(() => {
    if (loading && rows.length === 0) {
      return <Empty title={t('common.loading')} />
    }
    if (rows.length === 0) {
      return activeCount > 0 ? (
        <Empty title={t('timeline.noMatches')} hint={t('filter.clear')} />
      ) : (
        <Empty title={t('timeline.empty')} hint={t('timeline.emptyHint')} />
      )
    }
    return null
  }, [loading, rows.length, activeCount, t])

  return (
    <div className={css.wrap} style={{ ['--cols' as string]: COLUMNS }}>
      <div className={css.toolbar}>
        <div className={css.searchBox}>
          <Icon name="search" size={13} />
          <input
            className={css.searchInput}
            placeholder={t('timeline.search')}
            value={filters.search ?? ''}
            onChange={(e) => setFilter('search', e.target.value || null)}
          />
        </div>
        {loading ? <Spinner /> : null}
        <IconButton
          icon="filter"
          label={t('filter.quick')}
          active={showFilters || activeCount > 0}
          onClick={() => setShowFilters((v) => !v)}
        />
        {activeCount > 0 ? (
          <>
            <Badge tone="solid">{activeCount}</Badge>
            <IconButton icon="close" label={t('filter.clear')} onClick={clearFilters} />
          </>
        ) : null}
      </div>

      {showFilters ? <FilterBar /> : null}

      <div className={css.head} role="row">
        <span>{t('timeline.col.seq')}</span>
        <span>{t('timeline.col.method')}</span>
        <span>{t('timeline.col.status')}</span>
        <span>{t('timeline.col.host')}</span>
        <span>{t('timeline.col.path')}</span>
        <span>{t('timeline.col.type')}</span>
        <span style={{ textAlign: 'right' }}>{t('timeline.col.size')}</span>
        <span style={{ textAlign: 'right' }}>{t('timeline.col.time')}</span>
      </div>

      <div className={css.scroller} ref={scrollRef} onScroll={onScroll} role="rowgroup">
        {body ?? (
          <div className={css.list} style={{ height: virtualizer.getTotalSize() }}>
            {items.map((item) => (
              <TimelineRow
                key={rows[item.index].id}
                row={rows[item.index]}
                offset={item.start}
                selected={rows[item.index].id === selectedId}
                onSelect={select}
                onContextMenu={(event, row) => {
                  event.preventDefault()
                  void select(row.id)
                  setMenu({ row, position: { x: event.clientX, y: event.clientY } })
                }}
              />
            ))}
          </div>
        )}
      </div>

      <div className={css.footer}>
        <span>{t('timeline.showing', { shown: count(rows.length), total: count(total) })}</span>
        <span className={css.footerNote}>{t('timeline.orderNote')}</span>
        {capturing ? (
          <Toggle checked={live} onChange={setLive} label={t('state.capturing')} />
        ) : null}
      </div>

      {menu ? (
        <ContextMenu row={menu.row} position={menu.position} onClose={() => setMenu(null)} />
      ) : null}
    </div>
  )
}
