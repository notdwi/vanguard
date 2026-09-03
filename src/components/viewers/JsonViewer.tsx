import { useMemo, useState } from 'react'

import { useCopy } from '@/components/ui/CopyButton'
import { Icon } from '@/components/ui'
import { useApp } from '@/stores/app'

import css from './viewers.module.css'

type Json = unknown

interface NodeProps {
  name: string | null
  value: Json
  path: string
  depth: number
  filter: string
  defaultOpen: boolean
}

function typeOf(value: Json): string {
  if (value === null) return 'null'
  if (Array.isArray(value)) return 'array'
  return typeof value
}

function matches(value: Json, name: string | null, filter: string): boolean {
  if (!filter) return true
  const needle = filter.toLowerCase()
  if (name && name.toLowerCase().includes(needle)) return true
  const kind = typeOf(value)
  if (kind === 'object' || kind === 'array') {
    const entries =
      kind === 'array'
        ? (value as Json[]).map((v, i) => [String(i), v] as const)
        : Object.entries(value as Record<string, Json>)
    return entries.some(([k, v]) => matches(v, k, filter))
  }
  return String(value).toLowerCase().includes(needle)
}

function JsonNode({ name, value, path, depth, filter, defaultOpen }: NodeProps) {
  const t = useApp((s) => s.t)
  const copy = useCopy()
  const [open, setOpen] = useState(defaultOpen)
  const kind = typeOf(value)
  const branch = kind === 'object' || kind === 'array'

  if (!matches(value, name, filter)) return null

  const entries: [string, Json][] = branch
    ? kind === 'array'
      ? (value as Json[]).map((v, i) => [String(i), v])
      : Object.entries(value as Record<string, Json>)
    : []

  const childPath = (key: string) =>
    kind === 'array' ? `${path}[${key}]` : path ? `${path}.${key}` : `$.${key}`

  const scalar = () => {
    if (kind === 'string') return <span className={css.jsonString}>&quot;{String(value)}&quot;</span>
    if (kind === 'number') return <span className={css.jsonNumber}>{String(value)}</span>
    if (kind === 'boolean') return <span className={css.jsonKeyword}>{String(value)}</span>
    return <span className={css.jsonKeyword}>null</span>
  }

  return (
    <div className={css.jsonNode} style={{ paddingLeft: depth === 0 ? 0 : 12 }}>
      <div className={css.jsonRow}>
        {branch ? (
          <button
            type="button"
            className={css.jsonToggle}
            onClick={() => setOpen((v) => !v)}
            aria-expanded={open}
          >
            <Icon name={open ? 'chevronDown' : 'chevronRight'} size={11} />
          </button>
        ) : (
          <span className={css.jsonToggleSpacer} />
        )}

        {name != null ? <span className={css.jsonKey}>{name}</span> : null}

        {branch ? (
          <span className={css.jsonMeta}>
            {kind === 'array'
              ? t('json.items', { n: entries.length })
              : t('json.keys', { n: entries.length })}
          </span>
        ) : (
          scalar()
        )}

        <span className={css.jsonActions}>
          <button
            type="button"
            className={css.jsonAction}
            title={t('json.copyPath')}
            onClick={() => void copy(path || '$')}
          >
            path
          </button>
          <button
            type="button"
            className={css.jsonAction}
            title={t('json.copyValue')}
            onClick={() =>
              void copy(branch ? JSON.stringify(value, null, 2) : String(value ?? 'null'))
            }
          >
            <Icon name="copy" size={11} />
          </button>
        </span>
      </div>

      {branch && open ? (
        <div className={css.jsonChildren}>
          {entries.map(([key, child]) => (
            <JsonNode
              key={key}
              name={key}
              value={child}
              path={childPath(key)}
              depth={depth + 1}
              filter={filter}
              defaultOpen={depth < 1}
            />
          ))}
        </div>
      ) : null}
    </div>
  )
}

export function JsonViewer({ text }: { text: string }) {
  const t = useApp((s) => s.t)
  const [filter, setFilter] = useState('')

  const parsed = useMemo(() => {
    try {
      return { ok: true as const, value: JSON.parse(text) as Json }
    } catch (error) {
      return { ok: false as const, error: String(error) }
    }
  }, [text])

  if (!parsed.ok) {
    return <pre className={css.pre}>{text}</pre>
  }

  return (
    <div className={css.jsonWrap}>
      <div className={css.jsonFilter}>
        <Icon name="search" size={12} />
        <input
          className={css.jsonFilterInput}
          placeholder={t('json.filter')}
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
      </div>
      <div className={css.jsonTree}>
        <JsonNode
          name={null}
          value={parsed.value}
          path="$"
          depth={0}
          filter={filter.trim()}
          defaultOpen
        />
      </div>
    </div>
  )
}
