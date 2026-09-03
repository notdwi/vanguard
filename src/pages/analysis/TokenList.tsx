import { Badge, Empty } from '@/components/ui'
import { sequence } from '@/lib/format'
import { useApp } from '@/stores/app'
import type { DetectedToken } from '@/types/analysis'

import css from './analysis.module.css'

export function TokenList({ tokens }: { tokens: DetectedToken[] }) {
  const t = useApp((s) => s.t)

  if (tokens.length === 0) {
    return <Empty title={t('common.none')} />
  }

  return (
    <div className={css.tokenGrid}>
      {tokens.map((token) => (
        <div key={`${token.name}-${token.value_hash}`} className={css.tokenCard}>
          <div>
            <div className={css.tokenName}>{token.name}</div>
            <div className={css.tokenValue}>{token.value_preview}</div>
          </div>
          <div className={css.tokenMeta}>
            <Badge>{token.source}</Badge>
            <Badge tone="solid">{token.kind}</Badge>
            <span className={css.tokenValue}>
              {t('analysis.tokenUsedBy', { n: token.used_by.length })}
            </span>
          </div>
          <div className={css.tokenSequences}>
            {token.used_by.slice(0, 60).map((seq) => (
              <span key={seq} className={css.seqTag}>
                {sequence(seq)}
              </span>
            ))}
            {token.used_by.length > 60 ? (
              <span className={css.seqTag}>+{token.used_by.length - 60}</span>
            ) : null}
          </div>
        </div>
      ))}
    </div>
  )
}
