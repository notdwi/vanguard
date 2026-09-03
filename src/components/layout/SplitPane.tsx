import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react'

import css from '@/pages/pages.module.css'

interface SplitPaneProps {
  storageKey: string
  initial?: number
  min?: number
  max?: number
  left: ReactNode
  right: ReactNode
}

/// Horizontal split with a draggable divider; the ratio is remembered per key.
export function SplitPane({
  storageKey,
  initial = 0.52,
  min = 0.25,
  max = 0.75,
  left,
  right,
}: SplitPaneProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [ratio, setRatio] = useState(() => {
    try {
      const stored = localStorage.getItem(`vanguard.split.${storageKey}`)
      const parsed = stored ? Number(stored) : NaN
      return Number.isFinite(parsed) ? Math.min(max, Math.max(min, parsed)) : initial
    } catch {
      return initial
    }
  })
  const [dragging, setDragging] = useState(false)

  const persist = useCallback(
    (value: number) => {
      try {
        localStorage.setItem(`vanguard.split.${storageKey}`, String(value))
      } catch {
        /* ignore */
      }
    },
    [storageKey],
  )

  useEffect(() => {
    if (!dragging) return

    const onMove = (e: MouseEvent) => {
      const el = containerRef.current
      if (!el) return
      const rect = el.getBoundingClientRect()
      const next = Math.min(max, Math.max(min, (e.clientX - rect.left) / rect.width))
      setRatio(next)
    }
    const onUp = () => {
      setDragging(false)
      setRatio((current) => {
        persist(current)
        return current
      })
    }

    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    return () => {
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
  }, [dragging, min, max, persist])

  return (
    <div
      ref={containerRef}
      className={css.split}
      style={{
        ['--left' as string]: `${ratio * 100}%`,
        ['--right' as string]: `${(1 - ratio) * 100}%`,
      }}
    >
      <div className={css.pane}>{left}</div>
      <div
        className={`${css.divider} ${dragging ? css.dividerActive : ''}`}
        onMouseDown={() => setDragging(true)}
        role="separator"
        aria-orientation="vertical"
      />
      <div className={css.pane}>{right}</div>
    </div>
  )
}
