import { useEffect, useMemo, useRef, useState } from 'react'

import { Badge, Button, Empty, IconButton } from '@/components/ui'
import { useAnalysis } from '@/stores/analysis'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import { useTimeline } from '@/stores/timeline'
import type { LinkKind } from '@/types/analysis'

import { layoutGraph, NODE_HEIGHT, NODE_WIDTH } from './flow/layout'
import css from './flow/flow.module.css'
import page from './pages.module.css'

const PADDING = 48
const KINDS: LinkKind[] = [
  'cookie',
  'header-value',
  'query-value',
  'body-value',
  'json-value',
  'path-value',
]

export function FlowPage() {
  const t = useApp((s) => s.t)
  const setRoute = useApp((s) => s.setRoute)
  const session = useCapture((s) => s.session)
  const { bundle, loading, run } = useAnalysis()
  const select = useTimeline((s) => s.select)

  const containerRef = useRef<HTMLDivElement>(null)
  const [view, setView] = useState({ x: PADDING, y: PADDING, scale: 1 })
  const [dragging, setDragging] = useState(false)
  const dragStart = useRef({ x: 0, y: 0, viewX: 0, viewY: 0 })

  const layout = useMemo(
    () => (bundle ? layoutGraph(bundle.graph) : null),
    [bundle],
  )

  const fit = () => {
    const el = containerRef.current
    if (!el || !layout || layout.nodes.length === 0) return
    const scale = Math.min(
      1.4,
      Math.max(
        0.25,
        Math.min(
          (el.clientWidth - PADDING * 2) / Math.max(layout.width, 1),
          (el.clientHeight - PADDING * 2) / Math.max(layout.height, 1),
        ),
      ),
    )
    setView({
      scale,
      x: (el.clientWidth - layout.width * scale) / 2,
      y: PADDING,
    })
  }

  useEffect(fit, [layout])

  useEffect(() => {
    if (!dragging) return
    const onMove = (e: MouseEvent) => {
      setView((v) => ({
        ...v,
        x: dragStart.current.viewX + (e.clientX - dragStart.current.x),
        y: dragStart.current.viewY + (e.clientY - dragStart.current.y),
      }))
    }
    const onUp = () => setDragging(false)
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    return () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
  }, [dragging])

  if (!session) return <Empty title={t('capture.needSession')} />

  if (!bundle) {
    return (
      <Empty
        title={t('flow.empty')}
        hint={t('flow.emptyHint')}
        action={
          <Button variant="primary" icon="analysis" loading={loading} onClick={() => void run(session.id)}>
            {t('analysis.run')}
          </Button>
        }
      />
    )
  }

  if (!layout || layout.nodes.length === 0) {
    return (
      <Empty
        title={t('flow.empty')}
        hint={t('flow.emptyHint')}
        action={
          <Button icon="refresh" loading={loading} onClick={() => void run(session.id, true)}>
            {t('analysis.refresh')}
          </Button>
        }
      />
    )
  }

  const zoom = (factor: number) =>
    setView((v) => ({ ...v, scale: Math.min(2.5, Math.max(0.2, v.scale * factor)) }))

  const open = async (requestId: string) => {
    await select(requestId)
    setRoute('capture')
  }

  return (
    <div className={page.page}>
      <header className={page.pageHeader}>
        <h1 className={page.pageTitle}>{t('flow.title')}</h1>
        <Badge>{layout.nodes.length} nodes</Badge>
        <Badge>{layout.edges.length} links</Badge>
        <div className={page.pageSpacer} />
        <Button icon="refresh" loading={loading} onClick={() => void run(session.id, true)}>
          {t('analysis.refresh')}
        </Button>
      </header>

      <div
        ref={containerRef}
        className={`${css.canvas} ${dragging ? css.canvasDragging : ''}`}
        onMouseDown={(e) => {
          if (e.button !== 0) return
          dragStart.current = { x: e.clientX, y: e.clientY, viewX: view.x, viewY: view.y }
          setDragging(true)
        }}
        onWheel={(e) => {
          if (!e.ctrlKey && !e.metaKey) return
          e.preventDefault()
          zoom(e.deltaY < 0 ? 1.1 : 0.9)
        }}
      >
        <svg width="100%" height="100%">
          <g
            className={css.stage}
            transform={`translate(${view.x} ${view.y}) scale(${view.scale})`}
          >
            {layout.edges.map((edge, i) => (
              <g key={`${edge.from}-${edge.to}-${i}`}>
                <path
                  className={`${css.edge} ${edge.weight > 2 ? css.edgeStrong : ''}`}
                  d={edge.path}
                  markerEnd="url(#arrow)"
                />
                <text className={css.edgeLabel} x={edge.labelX} y={edge.labelY}>
                  {edge.label}
                </text>
              </g>
            ))}

            {layout.nodes.map((node) => (
              <g
                key={node.id}
                className={css.node}
                transform={`translate(${node.x} ${node.y})`}
                onClick={() => void open(node.sample_request_id)}
              >
                <rect
                  className={`${css.nodeBox} ${node.importance === 'high' ? css.nodeBoxHigh : ''}`}
                  width={NODE_WIDTH}
                  height={NODE_HEIGHT}
                />
                <text className={css.nodeMethod} x={12} y={18}>
                  {node.method}
                </text>
                <text className={css.nodeCount} x={NODE_WIDTH - 12} y={18}>
                  ×{node.count}
                </text>
                <text className={css.nodeLabel} x={12} y={34}>
                  {truncate(node.label, 28)}
                </text>
                <text className={css.nodeHost} x={12} y={46}>
                  {truncate(node.host, 34)}
                </text>
              </g>
            ))}
          </g>

          <defs>
            <marker
              id="arrow"
              viewBox="0 0 8 8"
              refX="7"
              refY="4"
              markerWidth="6"
              markerHeight="6"
              orient="auto-start-reverse"
            >
              <path d="M 0 1 L 7 4 L 0 7 z" fill="var(--fg-subtle)" />
            </marker>
          </defs>
        </svg>

        <div className={css.legend}>
          {KINDS.map((kind) => (
            <span key={kind} className={css.legendItem}>
              <span className={css.legendDash} />
              {t(`link.${kind}`)}
            </span>
          ))}
        </div>

        <div className={css.controls}>
          <IconButton icon="plus" label={t('flow.zoomIn')} onClick={() => zoom(1.15)} />
          <span className={css.zoomLabel}>{Math.round(view.scale * 100)}%</span>
          <IconButton icon="close" label={t('flow.zoomOut')} onClick={() => zoom(0.87)} />
          <IconButton icon="refresh" label={t('flow.fit')} onClick={fit} />
        </div>
      </div>
    </div>
  )
}

function truncate(value: string, max: number): string {
  return value.length <= max ? value : `${value.slice(0, max - 1)}…`
}
