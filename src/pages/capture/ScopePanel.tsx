import { useState } from 'react'

import { Field, Icon, Input, Select, Toggle } from '@/components/ui'
import { bytes } from '@/lib/format'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import type { CaptureConfig, ScopeMode } from '@/types/core'

import css from './capture.module.css'

type ListKey =
  | 'include_domains'
  | 'exclude_domains'
  | 'include_paths'
  | 'exclude_paths'

function TagList({
  values,
  placeholder,
  onAdd,
  onRemove,
}: {
  values: string[]
  placeholder: string
  onAdd: (value: string) => void
  onRemove: (value: string) => void
}) {
  const [draft, setDraft] = useState('')

  const commit = () => {
    const value = draft.trim()
    if (!value) return
    onAdd(value)
    setDraft('')
  }

  return (
    <>
      <div className={css.list}>
        {values.length === 0 ? <span className={css.empty}>—</span> : null}
        {values.map((value) => (
          <span key={value} className={css.tag}>
            {value}
            <button
              type="button"
              className={css.tagRemove}
              onClick={() => onRemove(value)}
              aria-label={`Remove ${value}`}
            >
              <Icon name="close" size={10} />
            </button>
          </span>
        ))}
      </div>
      <div className={css.addRow}>
        <Input
          mono
          value={draft}
          placeholder={placeholder}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault()
              commit()
            }
          }}
        />
      </div>
    </>
  )
}

export function ScopePanel() {
  const t = useApp((s) => s.t)
  const session = useCapture((s) => s.session)
  const updateConfig = useCapture((s) => s.updateConfig)
  if (!session) return null

  const config = session.config

  const patch = (next: Partial<CaptureConfig>) => {
    void updateConfig({ ...config, ...next })
  }

  const addTo = (key: ListKey, value: string) => {
    if (config[key].includes(value)) return
    patch({ [key]: [...config[key], value] } as Partial<CaptureConfig>)
  }

  const removeFrom = (key: ListKey, value: string) => {
    patch({ [key]: config[key].filter((v) => v !== value) } as Partial<CaptureConfig>)
  }

  return (
    <div className={css.scopePanel}>
      <div className={css.scopeGroup}>
        <span className={css.scopeTitle}>{t('capture.scope')}</span>
        <Field>
          <Select
            value={config.mode}
            onChange={(e) => patch({ mode: e.target.value as ScopeMode })}
          >
            <option value="all-traffic">{t('scope.allTraffic')}</option>
            <option value="domain-and-subdomains">{t('scope.domainAndSubdomains')}</option>
            <option value="exact-host">{t('scope.exactHost')}</option>
          </Select>
        </Field>
        <span className={css.scopeTitle}>{t('scope.bodies')}</span>
        <Toggle
          checked={config.capture_request_bodies}
          onChange={(v) => patch({ capture_request_bodies: v })}
          label={t('scope.captureRequestBodies')}
        />
        <Toggle
          checked={config.capture_response_bodies}
          onChange={(v) => patch({ capture_response_bodies: v })}
          label={t('scope.captureResponseBodies')}
        />
        <Field label={`${t('scope.maxBody')} — ${bytes(config.max_body_bytes)}`}>
          <Select
            value={String(config.max_body_bytes)}
            onChange={(e) => patch({ max_body_bytes: Number(e.target.value) })}
          >
            {[1, 4, 16, 64, 256].map((mb) => (
              <option key={mb} value={mb * 1024 * 1024}>
                {mb} MB
              </option>
            ))}
          </Select>
        </Field>
      </div>

      <div className={css.scopeGroup}>
        <span className={css.scopeTitle}>
          {t('scope.include')} · {t('scope.domains')}
        </span>
        <TagList
          values={config.include_domains}
          placeholder={t('scope.placeholderDomain')}
          onAdd={(v) => addTo('include_domains', v)}
          onRemove={(v) => removeFrom('include_domains', v)}
        />
        <span className={css.scopeTitle}>
          {t('scope.exclude')} · {t('scope.domains')}
        </span>
        <TagList
          values={config.exclude_domains}
          placeholder={t('scope.placeholderDomain')}
          onAdd={(v) => addTo('exclude_domains', v)}
          onRemove={(v) => removeFrom('exclude_domains', v)}
        />
      </div>

      <div className={css.scopeGroup}>
        <span className={css.scopeTitle}>
          {t('scope.include')} · {t('scope.paths')}
        </span>
        <TagList
          values={config.include_paths}
          placeholder={t('scope.placeholderPath')}
          onAdd={(v) => addTo('include_paths', v)}
          onRemove={(v) => removeFrom('include_paths', v)}
        />
        <span className={css.scopeTitle}>
          {t('scope.exclude')} · {t('scope.paths')}
        </span>
        <TagList
          values={config.exclude_paths}
          placeholder={t('scope.placeholderPath')}
          onAdd={(v) => addTo('exclude_paths', v)}
          onRemove={(v) => removeFrom('exclude_paths', v)}
        />
      </div>
    </div>
  )
}
