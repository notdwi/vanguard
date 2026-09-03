import { useEffect, useState } from 'react'

import { Field, Icon, Input, KeyValue, Select, Toggle } from '@/components/ui'
import { languages, type Language } from '@/i18n'
import { bytes } from '@/lib/format'
import { api } from '@/lib/ipc'
import { useApp, type Theme } from '@/stores/app'
import type { StorageInfo } from '@/types/analysis'

import css from './pages.module.css'

export function SettingsPage() {
  const t = useApp((s) => s.t)
  const settings = useApp((s) => s.settings)
  const saveSettings = useApp((s) => s.saveSettings)
  const theme = useApp((s) => s.theme)
  const setTheme = useApp((s) => s.setTheme)
  const [storage, setStorage] = useState<StorageInfo | null>(null)

  useEffect(() => {
    void api.storageInfo().then(setStorage).catch(() => undefined)
  }, [])

  return (
    <div className={css.page}>
      <header className={css.pageHeader}>
        <h1 className={css.pageTitle}>{t('settings.title')}</h1>
      </header>

      <div className={`${css.pageBody} ${css.narrow}`}>
        <div className={css.card}>
          <div className={css.cardHead}>
            <span className={css.cardTitle}>{t('app.name')}</span>
          </div>
          <div className={css.cardBody}>
            <div className={css.grid2}>
              <Field label={t('settings.language')}>
                <Select
                  value={settings.language}
                  onChange={(e) => void saveSettings({ language: e.target.value as Language })}
                >
                  {languages.map((l) => (
                    <option key={l.id} value={l.id}>
                      {l.label}
                    </option>
                  ))}
                </Select>
              </Field>

              <Field label={t('settings.theme')}>
                <Select value={theme} onChange={(e) => setTheme(e.target.value as Theme)}>
                  <option value="system">{t('settings.themeSystem')}</option>
                  <option value="light">{t('settings.themeLight')}</option>
                  <option value="dark">{t('settings.themeDark')}</option>
                </Select>
              </Field>

              <Field label={t('settings.proxyPort')}>
                <Input
                  mono
                  type="number"
                  min={1024}
                  max={65535}
                  value={settings.proxy_port}
                  onChange={(e) =>
                    void saveSettings({ proxy_port: Number(e.target.value) || 8080 })
                  }
                />
              </Field>

              <Field label={t('settings.pageSize')}>
                <Select
                  value={String(settings.timeline_page_size)}
                  onChange={(e) =>
                    void saveSettings({ timeline_page_size: Number(e.target.value) })
                  }
                >
                  {[200, 500, 1000, 2000, 5000].map((n) => (
                    <option key={n} value={n}>
                      {n}
                    </option>
                  ))}
                </Select>
              </Field>
            </div>

            <Toggle
              checked={settings.mask_secrets}
              onChange={(v) => void saveSettings({ mask_secrets: v })}
              label={t('settings.maskSecrets')}
            />
            <span className={css.cardNote}>{t('settings.maskHint')}</span>

            <Toggle
              checked={settings.auto_analyse}
              onChange={(v) => void saveSettings({ auto_analyse: v })}
              label={t('settings.autoAnalyse')}
            />
          </div>
        </div>

        <div className={css.card}>
          <div className={css.cardHead}>
            <span className={css.cardTitle}>{t('settings.storage')}</span>
          </div>
          <div className={css.cardBody}>
            <KeyValue
              items={[
                { key: t('settings.dataDir'), value: storage?.data_dir ?? '—' },
                {
                  key: t('settings.database'),
                  value: storage ? bytes(storage.database_bytes) : '—',
                },
                { key: t('settings.blobs'), value: storage ? bytes(storage.blob_bytes) : '—' },
              ]}
            />
          </div>
        </div>

        <div className={css.card}>
          <div className={css.cardHead}>
            <span className={css.cardTitle}>{t('settings.privacy')}</span>
          </div>
          <div className={css.cardBody}>
            <p className={css.cardNote}>{t('settings.privacyNote')}</p>
            <div className={css.note} style={{ marginBottom: 0 }}>
              <Icon name="warning" size={13} />
              <span>{t('disclaimer.short')}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
