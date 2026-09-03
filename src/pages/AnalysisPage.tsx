import { useEffect, useState } from 'react'

import { Badge, Button, Empty, Icon, Tabs } from '@/components/ui'
import { bytes, count, duration, sequence } from '@/lib/format'
import { api } from '@/lib/ipc'
import { useAnalysis } from '@/stores/analysis'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import { useTimeline } from '@/stores/timeline'

import { CookieTimeline } from './analysis/CookieTimeline'
import { RelationshipList } from './analysis/RelationshipList'
import { TokenList } from './analysis/TokenList'
import css from './pages.module.css'

type Tab = 'overview' | 'endpoints' | 'relationships' | 'tokens' | 'cookies'

export function AnalysisPage() {
  const t = useApp((s) => s.t)
  const setRoute = useApp((s) => s.setRoute)
  const autoAnalyse = useApp((s) => s.settings.auto_analyse)
  const session = useCapture((s) => s.session)
  const { bundle, loading, run } = useAnalysis()
  const select = useTimeline((s) => s.select)
  const [tab, setTab] = useState<Tab>('overview')

  useEffect(() => {
    if (session && autoAnalyse && !bundle) void run(session.id)
  }, [session, autoAnalyse, bundle, run])

  if (!session) return <Empty title={t('capture.needSession')} />

  if (!bundle) {
    return (
      <Empty
        title={t('analysis.run')}
        hint={t('analysis.runHint')}
        action={
          <Button variant="primary" icon="analysis" loading={loading} onClick={() => void run(session.id)}>
            {t('analysis.run')}
          </Button>
        }
      />
    )
  }

  const o = bundle.overview
  const stats = [
    { label: t('analysis.requests'), value: count(o.requests) },
    { label: t('analysis.domains'), value: count(o.domains) },
    { label: t('analysis.apiEndpoints'), value: count(o.api_endpoints) },
    { label: t('analysis.uniqueEndpoints'), value: count(o.unique_endpoints) },
    { label: t('analysis.jsonResponses'), value: count(o.json_responses) },
    { label: t('analysis.postRequests'), value: count(o.post_requests) },
    { label: t('analysis.withCookies'), value: count(o.with_cookies) },
    { label: t('analysis.tokens'), value: count(o.possible_tokens) },
    { label: t('analysis.high'), value: count(o.high_importance) },
    { label: t('analysis.errors'), value: count(o.errors) },
    { label: t('analysis.transferred'), value: bytes(o.total_bytes) },
  ]

  const openEndpoint = async (sequenceIds: number[]) => {
    const first = sequenceIds[0]
    if (first == null) return
    const { request_ids } = await api.endpointRequests(session.id, [first])
    if (request_ids[0]) {
      await select(request_ids[0])
      setRoute('capture')
    }
  }

  return (
    <div className={css.page}>
      <header className={css.pageHeader}>
        <h1 className={css.pageTitle}>{t('analysis.title')}</h1>
        <Badge>{session.name}</Badge>
        <div className={css.pageSpacer} />
        <Button icon="refresh" loading={loading} onClick={() => void run(session.id, true)}>
          {t('analysis.refresh')}
        </Button>
      </header>

      <Tabs
        items={[
          { id: 'overview', label: t('tab.overview') },
          { id: 'endpoints', label: t('analysis.endpoints'), count: bundle.endpoints.length },
          {
            id: 'relationships',
            label: t('analysis.relationships'),
            count: bundle.relationships.length,
          },
          { id: 'tokens', label: t('analysis.tokens'), count: bundle.tokens.length },
          { id: 'cookies', label: t('tab.cookies'), count: bundle.cookies.length },
        ]}
        active={tab}
        onSelect={(id) => setTab(id as Tab)}
      />

      <div className={css.pageBody}>
        {bundle.truncated ? (
          <div className={css.note}>
            <Icon name="warning" size={13} />
            <span>{t('analysis.truncated', { n: count(bundle.overview.requests) })}</span>
          </div>
        ) : null}

        {tab === 'overview' ? (
          <div className={css.statGrid}>
            {stats.map((s) => (
              <div key={s.label} className={css.stat}>
                <div className={css.statValue}>{s.value}</div>
                <div className={css.statLabel}>{s.label}</div>
              </div>
            ))}
          </div>
        ) : null}

        {tab === 'endpoints' ? (
          <div className={css.tableWrap}>
            <table className={css.dataTable}>
              <thead>
                <tr>
                  <th>{t('analysis.endpoint')}</th>
                  <th>{t('filter.host')}</th>
                  <th>{t('analysis.methodsCol')}</th>
                  <th>{t('analysis.statusCol')}</th>
                  <th className={css.numeric}>{t('analysis.avgCol')}</th>
                  <th className={css.numeric}>{t('analysis.requestsCol')}</th>
                </tr>
              </thead>
              <tbody>
                {bundle.endpoints.map((e) => (
                  <tr
                    key={`${e.host}${e.normalized}`}
                    className={css.clickableRow}
                    onClick={() => void openEndpoint(e.sequence_ids)}
                  >
                    <td className={css.monoCell}>
                      {e.is_api ? <Badge tone="solid">API</Badge> : null} {e.normalized}
                    </td>
                    <td className={css.monoCell}>{e.host}</td>
                    <td className={css.monoCell}>{e.methods.join(' ')}</td>
                    <td className={css.monoCell}>{e.status_codes.join(' ')}</td>
                    <td className={`${css.monoCell} ${css.numeric}`}>
                      {duration(e.avg_duration_ms)}
                    </td>
                    <td className={`${css.monoCell} ${css.numeric}`}>{count(e.count)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : null}

        {tab === 'relationships' ? (
          <RelationshipList
            links={bundle.relationships}
            onOpen={async (requestId) => {
              await select(requestId)
              setRoute('capture')
            }}
          />
        ) : null}

        {tab === 'tokens' ? <TokenList tokens={bundle.tokens} /> : null}

        {tab === 'cookies' ? (
          <CookieTimeline
            cookies={bundle.cookies}
            onOpen={async (requestId) => {
              await select(requestId)
              setRoute('capture')
            }}
          />
        ) : null}
      </div>
    </div>
  )
}

export { sequence }
