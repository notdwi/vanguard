import { useEffect, useRef } from 'react'
import { createPortal } from 'react-dom'

import { useCopy } from '@/components/ui/CopyButton'
import { api, errorMessage } from '@/lib/ipc'
import { useApp } from '@/stores/app'
import { useRepeater } from '@/stores/repeater'
import type { TimelineRow } from '@/types/core'

import css from './contextMenu.module.css'

export interface MenuPosition {
  x: number
  y: number
}

export function ContextMenu({
  row,
  position,
  onClose,
}: {
  row: TimelineRow
  position: MenuPosition
  onClose: () => void
}) {
  const t = useApp((s) => s.t)
  const notify = useApp((s) => s.notify)
  const setRoute = useApp((s) => s.setRoute)
  const maskSecrets = useApp((s) => s.settings.mask_secrets)
  const adoptDraft = useRepeater((s) => s.adoptDraft)
  const copy = useCopy()
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const onDown = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) onClose()
    }
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('mousedown', onDown)
    window.addEventListener('keydown', onKey)
    window.addEventListener('resize', onClose)
    return () => {
      window.removeEventListener('mousedown', onDown)
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('resize', onClose)
    }
  }, [onClose])

  const run = async (fn: () => Promise<void>) => {
    try {
      await fn()
    } catch (error) {
      notify(errorMessage(error), 'error')
    } finally {
      onClose()
    }
  }

  const url = `${row.scheme}://${row.host}${row.path}${row.query ? `?${row.query}` : ''}`

  const entries = [
    {
      label: t('action.copyUrl'),
      action: () => run(async () => void (await copy(url))),
    },
    {
      label: t('action.copyCurl'),
      action: () =>
        run(async () => {
          const command = await api.copyAsCurl(row.id, maskSecrets)
          await copy(command)
        }),
    },
    {
      label: t('action.copyHeaders'),
      action: () =>
        run(async () => {
          const detail = await api.requestDetail(row.id)
          const text = detail.request.request_headers
            .map((h) => `${h.name}: ${h.value}`)
            .join('\n')
          await copy(text)
        }),
    },
    {
      label: t('action.copyBody'),
      action: () =>
        run(async () => {
          const body = await api.loadBody(row.id, 'response', true)
          await copy(body.content ?? '')
        }),
    },
    { separator: true as const },
    {
      label: t('action.sendToRepeater'),
      action: () =>
        run(async () => {
          const draft = await api.sendToRepeater(row.id)
          await adoptDraft(draft)
          setRoute('repeater')
        }),
    },
  ]

  const style = {
    left: Math.min(position.x, window.innerWidth - 220),
    top: Math.min(position.y, window.innerHeight - 220),
  }

  return createPortal(
    <div className={css.menu} style={style} ref={ref} role="menu">
      {entries.map((entry, index) =>
        'separator' in entry ? (
          <div key={`sep-${index}`} className={css.separator} />
        ) : (
          <button key={entry.label} type="button" className={css.item} onClick={entry.action}>
            {entry.label}
          </button>
        ),
      )}
    </div>,
    document.body,
  )
}
