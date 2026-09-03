import { Badge, Icon, Select, Textarea } from '@/components/ui'
import { useApp } from '@/stores/app'
import { useRepeater } from '@/stores/repeater'

import { PairEditor } from './PairEditor'
import css from './repeater.module.css'

const METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']

export function RequestEditor() {
  const t = useApp((s) => s.t)
  const draft = useRepeater((s) => s.draft)
  const patchDraft = useRepeater((s) => s.patchDraft)
  if (!draft) return null

  return (
    <div className={css.paneBody}>
      {draft.source_sequence_id != null ? (
        <div className={css.warnNote}>
          <Icon name="warning" size={12} />
          <span>
            {t('repeater.fromCapture', { label: `#${draft.source_sequence_id}` })} ·{' '}
            {t('repeater.immutableNote')}
          </span>
        </div>
      ) : null}

      <section className={css.section}>
        <div className={css.sectionHead}>
          <span className={css.sectionTitle}>{t('tab.query')}</span>
          <div className={css.spacer} />
          <Badge>{draft.query.length}</Badge>
        </div>
        <PairEditor
          pairs={draft.query}
          onChange={(query) => patchDraft({ query })}
          addLabel={t('repeater.addQuery')}
          namePlaceholder="page"
          valuePlaceholder="2"
        />
      </section>

      <section className={css.section}>
        <div className={css.sectionHead}>
          <span className={css.sectionTitle}>{t('tab.headers')}</span>
          <div className={css.spacer} />
          <Badge>{draft.headers.length}</Badge>
        </div>
        <PairEditor
          pairs={draft.headers}
          onChange={(headers) => patchDraft({ headers })}
          addLabel={t('repeater.addHeader')}
          namePlaceholder="accept"
          valuePlaceholder="application/json"
        />
      </section>

      <section className={css.section}>
        <div className={css.sectionHead}>
          <span className={css.sectionTitle}>{t('tab.cookies')}</span>
          <div className={css.spacer} />
          <Badge>{draft.cookies.length}</Badge>
        </div>
        <PairEditor
          pairs={draft.cookies}
          onChange={(cookies) => patchDraft({ cookies })}
          addLabel={t('repeater.addCookie')}
          namePlaceholder="session_id"
        />
      </section>

      <section className={css.section}>
        <div className={css.sectionHead}>
          <span className={css.sectionTitle}>{t('tab.body')}</span>
        </div>
        <Textarea
          value={draft.body}
          rows={10}
          placeholder="{}"
          onChange={(e) => patchDraft({ body: e.target.value })}
        />
      </section>
    </div>
  )
}

export function MethodSelect() {
  const draft = useRepeater((s) => s.draft)
  const patchDraft = useRepeater((s) => s.patchDraft)
  if (!draft) return null

  return (
    <Select
      className={css.methodSelect}
      value={draft.method}
      onChange={(e) => patchDraft({ method: e.target.value })}
    >
      {METHODS.map((m) => (
        <option key={m} value={m}>
          {m}
        </option>
      ))}
    </Select>
  )
}
