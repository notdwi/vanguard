import { useEffect, useState } from 'react'

import { Button, Empty, IconButton, Input } from '@/components/ui'
import { CopyButton } from '@/components/ui/CopyButton'
import { api } from '@/lib/ipc'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import { useRepeater } from '@/stores/repeater'
import type { ReplayResult } from '@/types/repeater'

import { CompareDialog } from './repeater/CompareDialog'
import { MethodSelect, RequestEditor } from './repeater/RequestEditor'
import { ReplayHistory } from './repeater/ReplayHistory'
import { ResponsePane } from './repeater/ResponsePane'
import { RunBar } from './repeater/RunBar'
import css from './repeater/repeater.module.css'

export function RepeaterPage() {
  const t = useApp((s) => s.t)
  const masked = useApp((s) => s.settings.mask_secrets)
  const session = useCapture((s) => s.session)
  const {
    drafts,
    draft,
    activeId,
    results,
    loadDrafts,
    selectDraft,
    createDraft,
    removeDraft,
    patchDraft,
    persistDraft,
  } = useRepeater()

  const [selected, setSelected] = useState<ReplayResult | null>(null)
  const [comparing, setComparing] = useState(false)

  useEffect(() => {
    if (session) void loadDrafts(session.id)
  }, [session, loadDrafts])

  useEffect(() => {
    setSelected(results[0] ?? null)
  }, [results])

  if (!session) {
    return <Empty title={t('capture.needSession')} />
  }

  return (
    <div className={css.layout}>
      <aside className={css.sidebar}>
        <div className={css.sidebarHead}>
          <span className={css.sidebarTitle}>{t('repeater.drafts')}</span>
          <IconButton
            icon="plus"
            label={t('repeater.newDraft')}
            onClick={() => void createDraft(session.id)}
          />
        </div>
        <div className={css.draftList}>
          {drafts.length === 0 ? (
            <Empty title={t('repeater.empty')} hint={t('repeater.emptyHint')} />
          ) : (
            drafts.map((d) => (
              <button
                key={d.id}
                type="button"
                className={`${css.draftItem} ${activeId === d.id ? css.draftItemActive : ''}`}
                onClick={() => void selectDraft(d.id)}
              >
                <span className={css.draftLabel}>{d.label}</span>
                <span className={css.draftMeta}>{d.url}</span>
              </button>
            ))
          )}
        </div>
      </aside>

      <div className={css.paneDivider} />

      {draft ? (
        <div className={css.main}>
          <div className={css.urlBar}>
            <MethodSelect />
            <Input
              className={css.urlInput}
              value={draft.url}
              onChange={(e) => patchDraft({ url: e.target.value })}
              onBlur={() => void persistDraft()}
              placeholder="https://api.site.com/search"
            />
            <CopyButton
              value={() => api.draftAsCurl(draft, masked)}
              label={t('action.copyCurl')}
            />
            <Button small icon="trash" onClick={() => void removeDraft(draft.id)}>
              {t('repeater.deleteDraft')}
            </Button>
          </div>

          <div className={css.panes}>
            <div className={css.pane}>
              <RequestEditor />
              <RunBar />
            </div>
            <div className={css.paneDivider} />
            <div className={css.pane}>
              <ResponsePane result={selected} />
              <ReplayHistory
                selectedId={selected?.id ?? null}
                onSelect={setSelected}
                onCompare={() => setComparing(true)}
              />
            </div>
          </div>
        </div>
      ) : (
        <Empty
          title={t('repeater.empty')}
          hint={t('repeater.emptyHint')}
          action={
            <Button variant="primary" icon="plus" onClick={() => void createDraft(session.id)}>
              {t('repeater.newDraft')}
            </Button>
          }
        />
      )}

      <CompareDialog open={comparing} onClose={() => setComparing(false)} />
    </div>
  )
}
