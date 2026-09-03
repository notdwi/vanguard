import { useEffect } from 'react'

import { Badge, Empty, Icon, Spinner } from '@/components/ui'
import { sequence } from '@/lib/format'
import { useAnalysis } from '@/stores/analysis'
import { useApp } from '@/stores/app'
import { useTimeline } from '@/stores/timeline'
import type { Relationship } from '@/types/analysis'
import type { CapturedRequest } from '@/types/core'

import css from './inspector.module.css'

function LinkCard({ link, direction }: { link: Relationship; direction: 'in' | 'out' }) {
  const t = useApp((s) => s.t)
  const select = useTimeline((s) => s.select)
  const other = direction === 'in' ? link.from_request_id : link.to_request_id
  const otherSeq = direction === 'in' ? link.from_sequence_id : link.to_sequence_id
  const otherPath = direction === 'in' ? link.from_path : link.to_path

  return (
    <div className={css.link}>
      <div className={css.linkChain}>
        <button type="button" className={css.seqLink} onClick={() => void select(other)}>
          {sequence(otherSeq)}
        </button>
        <span className={css.linkPath}>{otherPath}</span>
        <Icon name="arrowRight" size={12} />
        <span className={css.linkValue}>{link.value_preview}</span>
      </div>
      <Badge title={t(`link.${link.kind}`)}>{t(`link.${link.kind}`)}</Badge>
      <div className={css.linkPath}>
        {link.source_json_path ? `${link.source_json_path} → ` : ''}
        {link.target_location}
      </div>
    </div>
  )
}

export function AnalysisTab({ request }: { request: CapturedRequest }) {
  const t = useApp((s) => s.t)
  const analysis = useAnalysis((s) => s.requestAnalysis[request.id])
  const forRequest = useAnalysis((s) => s.forRequest)

  useEffect(() => {
    void forRequest(request.session_id, request.id)
  }, [request.session_id, request.id, forRequest])

  if (!analysis) {
    return (
      <div className={css.section}>
        <Spinner />
      </div>
    )
  }

  return (
    <div>
      <div className={css.noteBox}>
        <Icon name="warning" size={13} />
        <span>{t('analysis.heuristicNote')}</span>
      </div>

      <section className={css.section}>
        <div className={css.sectionHead}>
          <span className={css.sectionTitle}>{t('field.endpoint')}</span>
          <div className={css.sectionSpacer} />
          {analysis.is_api ? <Badge tone="solid">API</Badge> : null}
          <Badge>{t('analysis.repeat', { n: analysis.repeat_count })}</Badge>
        </div>
        <p className={css.valueCell}>{analysis.normalized_endpoint}</p>
      </section>

      {analysis.reasons.length > 0 ? (
        <section className={css.section}>
          <div className={css.sectionHead}>
            <span className={css.sectionTitle}>{t('analysis.whyImportant')}</span>
            <div className={css.sectionSpacer} />
            <Badge tone={analysis.importance === 'high' ? 'solid' : 'default'}>
              {t(`importance.${analysis.importance as 'high' | 'medium' | 'low'}`)}
            </Badge>
          </div>
          <ul className={css.reasonList}>
            {analysis.reasons.map((r) => (
              <li key={r} className={css.reason}>
                {r}
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      {analysis.tokens.length > 0 ? (
        <section className={css.section}>
          <div className={css.sectionHead}>
            <span className={css.sectionTitle}>{t('analysis.tokens')}</span>
          </div>
          <table className={css.table}>
            <tbody>
              {analysis.tokens.map((token) => (
                <tr key={`${token.name}-${token.value_hash}`}>
                  <td className={css.nameCell}>{token.name}</td>
                  <td className={css.valueCell}>{token.value_preview}</td>
                  <td className={css.rowActions}>
                    <Badge>{token.kind}</Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      ) : null}

      {analysis.detected_ids.length > 0 ? (
        <section className={css.section}>
          <div className={css.sectionHead}>
            <span className={css.sectionTitle}>{t('analysis.detectedIds')}</span>
          </div>
          <table className={css.table}>
            <tbody>
              {analysis.detected_ids.slice(0, 40).map((id, i) => (
                <tr key={`${id.location}-${i}`}>
                  <td className={css.nameCell}>{id.location}</td>
                  <td className={css.valueCell}>{id.value}</td>
                  <td className={css.rowActions}>
                    <Badge>{id.kind}</Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      ) : null}

      <section className={css.section}>
        <div className={css.sectionHead}>
          <span className={css.sectionTitle}>{t('analysis.inbound')}</span>
          <div className={css.sectionSpacer} />
          <Badge>{analysis.inbound.length}</Badge>
        </div>
        {analysis.inbound.length === 0 ? (
          <p className={css.reason}>{t('analysis.noRelationships')}</p>
        ) : (
          analysis.inbound.map((link, i) => (
            <LinkCard key={`in-${i}`} link={link} direction="in" />
          ))
        )}
      </section>

      <section className={css.section}>
        <div className={css.sectionHead}>
          <span className={css.sectionTitle}>{t('analysis.outbound')}</span>
          <div className={css.sectionSpacer} />
          <Badge>{analysis.outbound.length}</Badge>
        </div>
        {analysis.outbound.length === 0 ? (
          <Empty title={t('analysis.noRelationships')} />
        ) : (
          analysis.outbound
            .slice(0, 40)
            .map((link, i) => <LinkCard key={`out-${i}`} link={link} direction="out" />)
        )}
      </section>
    </div>
  )
}
