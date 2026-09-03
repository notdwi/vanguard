import { useCallback, useEffect, useRef, useState } from 'react'

import { writeText } from '@tauri-apps/plugin-clipboard-manager'

import { useApp } from '@/stores/app'

import { IconButton } from './index'

export function useCopy() {
  const notify = useApp((s) => s.notify)
  const t = useApp((s) => s.t)

  return useCallback(
    async (value: string) => {
      try {
        await writeText(value)
        return true
      } catch {
        try {
          await navigator.clipboard.writeText(value)
          return true
        } catch {
          notify(t('common.error'), 'error')
          return false
        }
      }
    },
    [notify, t],
  )
}

export function CopyButton({
  value,
  label,
  size = 13,
}: {
  value: string | (() => Promise<string> | string)
  label?: string
  size?: number
}) {
  const copy = useCopy()
  const t = useApp((s) => s.t)
  const [done, setDone] = useState(false)
  const timer = useRef<number | null>(null)

  useEffect(
    () => () => {
      if (timer.current) window.clearTimeout(timer.current)
    },
    [],
  )

  const onClick = async () => {
    const resolved = typeof value === 'function' ? await value() : value
    if (!(await copy(resolved))) return
    setDone(true)
    if (timer.current) window.clearTimeout(timer.current)
    timer.current = window.setTimeout(() => setDone(false), 1400)
  }

  return (
    <IconButton
      icon={done ? 'check' : 'copy'}
      size={size}
      label={done ? t('action.copied') : label ?? t('action.copy')}
      onClick={onClick}
    />
  )
}
