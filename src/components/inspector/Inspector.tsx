import { useMemo, useState } from 'react'

import { Badge, Button, Empty, Tabs, type TabItem } from '@/components/ui'
import { CopyButton } from '@/components/ui/CopyButton'
import { BodyViewer } from '@/components/viewers/BodyViewer'
import { sequence } from '@/lib/format'
import { api, errorMessage } from '@/lib/ipc'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import { useRepeater } from '@/stores/repeater'
import { useTimeline } from '@/stores/timeline'

import { AnalysisTab } from './AnalysisTab'
import { CookiesTab } from './CookiesTab'
import { HeaderTable } from './HeaderTable'
import { OverviewTab } from './OverviewTab'
import { RawTab } from './RawTab'
import css from './inspector.module.css'

type TabId =
  | 'overview'
  | 'headers'
  | 'cookies'
  | 'query'
  | 'body'
  | 'response'
  | 'analysis'
  | 'raw'

export function Inspector() {
  const t = useApp((s) => s.t)
  const notify = useApp((s) => s.notify)
  const setRoute = useApp((s) => s.setRoute)
  const masked = useApp((s) => s.settings.mask_secrets)
  const detail = useTimeline((s) => s.detail)
  const session = useCapture((s) => s.session)
  const adoptDraft = useRepeater((s) => s.adoptDraft)
  const [tab, setTab] = useState<TabId>('overview')

  const maxBody = session?.config.max_body_bytes ?? 16 * 1024 * 1024

  const tabs = useMemo<TabItem[]>(() => {
    if (!detail) return []
    const r = detail.request
    return [
      { id: 'overview', label: t('tab.overview') },
      { id: 'headers', label: t('tab.headers'), count: r.request_headers.length },
      {
        id: 'cookies',
        label: t('tab.cookies'),
        count: detail.request_cookies.length + detail.response_cookies.length,
      },
      { id: 'query', label: t('tab.query'), count: detail.query.length },
      { id: 'body', label: t('tab.body'), count: r.request_body.size > 0 ? 1 : 0 },
      { id: 'response', label: t('tab.response') },
      { id: 'analysis', label: t('tab.analysis') },
      { id: 'raw', label: t('tab.raw') },
    ]
  }, [detail, t])

  if (!detail) {
    return (
      <div className={css.wrap}>
        <Empty title={t('inspector.empty')} hint={t('inspector.emptyHint')} />
      </div>
    )
  }

  const r = detail.request
  const sendToRepeater = async () => {
    try {
      const draft = await api.sendToRepeater(r.id)
      await adoptDraft(draft)
      setRoute('repeater')
    } catch (error) {
      notify(errorMessage(error), 'error')
    }
  }

  const flushTab = tab === 'body' || tab === 'response' || tab === 'raw'

  return (
    <div className={css.wrap}>
      <header className={css.header}>
        <div className={css.headline}>
          <span className={css.seq}>{sequence(r.sequence_id)}</span>
          <span className={css.method}>{r.method}</span>
          <span className={css.originHost}>{r.host}</span>
          <span className={css.url} title={r.url}>
            {r.path}
            {r.query ? <span className={css.urlQuery}>?{r.query}</span> : null}
          </span>
        </div>
        <div className={css.actions}>
          {r.response ? <Badge>{r.response.status}</Badge> : null}
          <CopyButton value={r.url} label={t('action.copyUrl')} />
          <CopyButton
            value={() => api.copyAsCurl(r.id, masked)}
            label={t('action.copyCurl')}
          />
          <Button small icon="repeater" onClick={sendToRepeater}>
            {t('action.sendToRepeater')}
          </Button>
        </div>
      </header>

      <Tabs items={tabs} active={tab} onSelect={(id) => setTab(id as TabId)} />

      <div className={flushTab ? css.flush : css.panel}>
        {tab === 'overview' ? (
          <div className={css.scroll}>
            <OverviewTab detail={detail} />
          </div>
        ) : null}

        {tab === 'headers' ? (
          <div className={css.scroll}>
            <section className={css.section}>
              <div className={css.sectionHead}>
                <span className={css.sectionTitle}>{t('headers.request')}</span>
                <div className={css.sectionSpacer} />
                <Badge>{r.request_headers.length}</Badge>
              </div>
              <HeaderTable headers={r.request_headers} masked={masked} />
            </section>
            <section className={css.section}>
              <div className={css.sectionHead}>
                <span className={css.sectionTitle}>{t('headers.response')}</span>
                <div className={css.sectionSpacer} />
                <Badge>{r.response?.headers.length ?? 0}</Badge>
              </div>
              <HeaderTable headers={r.response?.headers ?? []} masked={masked} />
            </section>
          </div>
        ) : null}

        {tab === 'cookies' ? (
          <div className={css.scroll}>
            <CookiesTab detail={detail} />
          </div>
        ) : null}

        {tab === 'query' ? (
          <div className={css.scroll}>
            {detail.query.length === 0 ? (
              <p className={css.reason}>{t('query.none')}</p>
            ) : (
              <table className={css.table}>
                <thead>
                  <tr>
                    <th>{t('query.parameter')}</th>
                    <th>{t('query.value')}</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {detail.query.map((p, i) => (
                    <tr key={`${p.name}-${i}`}>
                      <td className={css.nameCell}>{p.name}</td>
                      <td className={css.valueCell}>{p.value}</td>
                      <td className={css.rowActions}>
                        <CopyButton value={p.value} size={12} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        ) : null}

        {tab === 'body' ? (
          <BodyViewer
            requestId={r.id}
            side="request"
            reference={r.request_body}
            maxBodyBytes={maxBody}
          />
        ) : null}

        {tab === 'response' ? (
          r.response ? (
            <BodyViewer
              requestId={r.id}
              side="response"
              reference={r.response.body}
              maxBodyBytes={maxBody}
            />
          ) : (
            <Empty title={t('repeater.noResponse')} hint={r.error ?? undefined} />
          )
        ) : null}

        {tab === 'analysis' ? (
          <div className={css.scroll}>
            <AnalysisTab request={r} />
          </div>
        ) : null}

        {tab === 'raw' ? <RawTab detail={detail} /> : null}
      </div>
    </div>
  )
}
