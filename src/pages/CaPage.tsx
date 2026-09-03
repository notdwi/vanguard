import { useEffect, useState } from 'react'

import { save as saveDialog } from '@tauri-apps/plugin-dialog'

import { Badge, Button, Icon, KeyValue, StatusDot } from '@/components/ui'
import { CopyButton } from '@/components/ui/CopyButton'
import { ConfirmDialog } from '@/components/ui/Modal'
import { api, errorMessage } from '@/lib/ipc'
import { useApp } from '@/stores/app'
import type { CaInfo, TrustStorePlan } from '@/types/analysis'

import css from './pages.module.css'

export function CaPage() {
  const t = useApp((s) => s.t)
  const notify = useApp((s) => s.notify)
  const [info, setInfo] = useState<CaInfo | null>(null)
  const [plan, setPlan] = useState<TrustStorePlan | null>(null)
  const [busy, setBusy] = useState(false)
  const [confirming, setConfirming] = useState<'install' | 'delete' | null>(null)

  const refresh = async () => {
    try {
      setInfo(await api.caInfo())
      setPlan(await api.caPlan())
    } catch (error) {
      notify(errorMessage(error), 'error')
    }
  }

  useEffect(() => {
    void refresh()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const act = async (fn: () => Promise<CaInfo>) => {
    setBusy(true)
    try {
      setInfo(await fn())
    } catch (error) {
      notify(errorMessage(error), 'error')
    } finally {
      setBusy(false)
      setConfirming(null)
    }
  }

  const exportCert = async () => {
    try {
      const path = await saveDialog({
        defaultPath: 'vanguard-ca.crt',
        filters: [{ name: 'Certificate', extensions: ['crt', 'pem'] }],
      })
      if (!path) return
      await api.exportCa(path)
      notify(t('sessions.exported', { path }))
    } catch (error) {
      notify(errorMessage(error), 'error')
    }
  }

  return (
    <div className={css.page}>
      <header className={css.pageHeader}>
        <h1 className={css.pageTitle}>{t('ca.title')}</h1>
        <div className={css.pageSpacer} />
        {info ? (
          <Badge tone={info.installed ? 'solid' : 'default'}>
            <StatusDot live={info.installed} />
            {info.installed ? t('ca.installed') : t('ca.notInstalled')}
          </Badge>
        ) : null}
      </header>

      <div className={`${css.pageBody} ${css.narrow}`}>
        <p className={css.cardNote} style={{ marginBottom: 'var(--space-6)' }}>
          {t('ca.intro')}
        </p>

        <div className={css.note}>
          <Icon name="warning" size={13} />
          <span>{t('ca.warning')}</span>
        </div>

        <div className={css.card}>
          <div className={css.cardHead}>
            <span className={css.cardTitle}>{t('nav.certificate')}</span>
          </div>
          <div className={css.cardBody}>
            {info?.exists ? (
              <KeyValue
                items={[
                  { key: t('common.name'), value: info.common_name },
                  {
                    key: t('ca.fingerprint'),
                    value: (
                      <span style={{ display: 'inline-flex', gap: 6, alignItems: 'center' }}>
                        <span style={{ wordBreak: 'break-all' }}>{info.fingerprint ?? '—'}</span>
                        {info.fingerprint ? <CopyButton value={info.fingerprint} size={12} /> : null}
                      </span>
                    ),
                  },
                  { key: t('ca.certPath'), value: info.cert_path },
                  { key: t('ca.keyPath'), value: info.key_path },
                ]}
              />
            ) : (
              <p className={css.cardNote}>{t('ca.missing')}</p>
            )}

            <div className={css.row}>
              {info?.exists ? null : (
                <Button variant="primary" icon="certificate" loading={busy} onClick={() => act(api.generateCa)}>
                  {t('ca.generate')}
                </Button>
              )}
              {info?.installed ? (
                <Button icon="close" loading={busy} onClick={() => act(api.uninstallCa)}>
                  {t('ca.uninstall')}
                </Button>
              ) : (
                <Button
                  variant="primary"
                  icon="check"
                  loading={busy}
                  onClick={() => setConfirming('install')}
                >
                  {t('ca.install')}
                </Button>
              )}
              <Button icon="download" onClick={exportCert}>
                {t('ca.export')}
              </Button>
              {info?.exists ? (
                <Button icon="trash" onClick={() => setConfirming('delete')}>
                  {t('ca.delete')}
                </Button>
              ) : null}
            </div>
          </div>
        </div>

        {plan ? (
          <div className={css.card}>
            <div className={css.cardHead}>
              <span className={css.cardTitle}>{t('ca.whatHappens')}</span>
              {plan.requires_elevation ? <Badge>{t('ca.elevation')}</Badge> : null}
            </div>
            <div className={css.cardBody}>
              <pre className={css.monoCell} style={{ margin: 0, whiteSpace: 'pre-wrap' }}>
                {plan.steps.join('\n')}
              </pre>
              <div>
                <p className={css.sectionTitle}>{t('ca.manual')}</p>
                <ol className={css.cardNote} style={{ paddingLeft: 18, margin: 0 }}>
                  {plan.manual_instructions.map((step) => (
                    <li key={step} style={{ marginBottom: 4 }}>
                      {step}
                    </li>
                  ))}
                </ol>
              </div>
            </div>
          </div>
        ) : null}
      </div>

      <ConfirmDialog
        open={confirming !== null}
        title={confirming === 'delete' ? t('ca.delete') : t('ca.install')}
        message={confirming === 'delete' ? t('ca.confirmDelete') : t('ca.confirmInstall')}
        destructive={confirming === 'delete'}
        onConfirm={() => act(confirming === 'delete' ? api.deleteCa : api.installCa)}
        onCancel={() => setConfirming(null)}
      />
    </div>
  )
}
