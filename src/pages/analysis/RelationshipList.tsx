import { Badge, Empty, Icon } from '@/components/ui'
import { sequence } from '@/lib/format'
import { useApp } from '@/stores/app'
import type { Relationship } from '@/types/analysis'

import css from '../pages.module.css'
import local from './analysis.module.css'

export function RelationshipList({
  links,
  onOpen,
}: {
  links: Relationship[]
  onOpen: (requestId: string) => void
}) {
  const t = useApp((s) => s.t)

  if (links.length === 0) {
    return <Empty title={t('analysis.noRelationships')} hint={t('analysis.heuristicNote')} />
  }

  return (
    <>
      <div className={css.note}>
        <Icon name="warning" size={13} />
        <span>{t('analysis.heuristicNote')}</span>
      </div>

      <div className={local.linkList}>
        {links.slice(0, 300).map((link, i) => (
          <div key={i} className={local.linkCard}>
            <div className={local.linkChain}>
              <button
                type="button"
                className={local.seqLink}
                onClick={() => onOpen(link.from_request_id)}
              >
                {sequence(link.from_sequence_id)}
              </button>
              <span className={local.linkPath}>{link.from_path}</span>
              <Icon name="arrowRight" size={13} />
              <button
                type="button"
                className={local.seqLink}
                onClick={() => onOpen(link.to_request_id)}
              >
                {sequence(link.to_sequence_id)}
              </button>
              <span className={local.linkPath}>{link.to_path}</span>
              <div className={local.spacer} />
              <Badge>{t(`link.${link.kind}`)}</Badge>
            </div>
            <div className={local.linkDetail}>
              <span className={local.linkValue}>{link.value_preview}</span>
              <span className={local.linkPath}>
                {link.source_json_path ?? '—'} → {link.target_location}
              </span>
            </div>
          </div>
        ))}
      </div>
    </>
  )
}
