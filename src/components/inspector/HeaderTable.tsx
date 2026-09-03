import { useState } from 'react'

import { IconButton } from '@/components/ui'
import { CopyButton } from '@/components/ui/CopyButton'
import { useApp } from '@/stores/app'
import type { Header } from '@/types/core'

import css from './inspector.module.css'

const SENSITIVE = [
  'authorization',
  'proxy-authorization',
  'cookie',
  'set-cookie',
  'x-api-key',
  'x-auth-token',
  'x-csrf-token',
  'x-xsrf-token',
  'api-key',
]

export function isSensitive(name: string): boolean {
  const lower = name.toLowerCase()
  return (
    SENSITIVE.includes(lower) ||
    lower.includes('token') ||
    lower.includes('secret') ||
    lower.includes('password')
  )
}

export function maskValue(value: string): string {
  const trimmed = value.trim()
  if (!trimmed) return ''
  const [scheme, ...rest] = trimmed.split(' ')
  const known = ['bearer', 'basic', 'digest', 'token']
  if (rest.length > 0 && known.includes(scheme.toLowerCase())) {
    return `${scheme} ${stars(rest.join(' '))}`
  }
  return stars(trimmed)
}

function stars(value: string): string {
  const head = value.length > 12 ? value.slice(0, 4) : ''
  return `${head}${'*'.repeat(8)}`
}

function HeaderRow({ header, masked }: { header: Header; masked: boolean }) {
  const t = useApp((s) => s.t)
  const [revealed, setRevealed] = useState(false)
  const hide = masked && isSensitive(header.name) && !revealed

  return (
    <tr>
      <td className={css.nameCell}>{header.name}</td>
      <td className={`${css.valueCell} ${hide ? css.maskedValue : ''}`}>
        {hide ? maskValue(header.value) : header.value}
      </td>
      <td className={css.rowActions}>
        {masked && isSensitive(header.name) ? (
          <IconButton
            icon={revealed ? 'eyeOff' : 'eye'}
            size={12}
            label={revealed ? t('headers.hide') : t('headers.reveal')}
            onClick={() => setRevealed((v) => !v)}
          />
        ) : null}
        <CopyButton value={header.value} size={12} />
      </td>
    </tr>
  )
}

export function HeaderTable({ headers, masked }: { headers: Header[]; masked: boolean }) {
  const t = useApp((s) => s.t)

  if (headers.length === 0) {
    return <p className={css.reason}>{t('headers.none')}</p>
  }

  return (
    <table className={css.table}>
      <thead>
        <tr>
          <th>{t('common.name')}</th>
          <th>{t('common.value')}</th>
          <th />
        </tr>
      </thead>
      <tbody>
        {headers.map((h, i) => (
          <HeaderRow key={`${h.name}-${i}`} header={h} masked={masked} />
        ))}
      </tbody>
    </table>
  )
}
