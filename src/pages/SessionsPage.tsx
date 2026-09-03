import { useEffect, useState } from 'react'

import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'

import { Badge, Button, Empty, IconButton, Input, StatusDot } from '@/components/ui'
import { ConfirmDialog } from '@/components/ui/Modal'
import { count, dateLabel, relativeDay } from '@/lib/format'
import { api, errorMessage } from '@/lib/ipc'
import { useAnalysis } from '@/stores/analysis'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import { useTimeline } from '@/stores/timeline'

import css from './pages.module.css'
import local from './sessions.module.css'

type Pending = { kind: 'delete' | 'clear'; id: string } | null

export function SessionsPage() {
  const t = useApp((s) => s.t)
  const notify = useApp((s) => s.notify)
  const setRoute = useApp((s) => s.setRoute)
  const { sessions, session, refreshSessions, openSession, createSession, renameSession, deleteSession, clearSession } =
    useCapture()
  const resetTimeline = useTimeline((s) => s.reset)
  const resetAnalysis = useAnalysis((s) => s.reset)

  const [name, setName] = useState('')
  const [renaming, setRenaming] = useState<string | null>(null)
  const [renameValue, setRenameValue] = useState('')
  const [pending, setPending] = useState<Pending>(null)

  useEffect(() => {
    void refreshSessions()
  }, [refreshSessions])

  const create = async () => {
    const created = await createSession(name)
    setName('')
    if (created) {
      resetTimeline()
      resetAnalysis()
      setRoute('capture')
    }
  }

  const open = async (id: string) => {
    resetTimeline()
    resetAnalysis()
    await openSession(id)
    setRoute('capture')
  }

  const importHar = async () => {
    try {
      const path = await openDialog({
        multiple: false,
        filters: [{ name: 'HAR', extensions: ['har', 'json'] }],
      })
      if (typeof path !== 'string') return
      const report = await api.importHar(path)
      notify(t('sessions.imported', { n: report.imported, skipped: report.skipped }))
      await refreshSessions()
      await open(report.session_id)
    } catch (error) {
      notify(errorMessage(error), 'error')
    }
  }

  const exportHar = async (id: string, label: string) => {
    try {
      const path = await saveDialog({
        defaultPath: `${label.replace(/[^\w.-]+/g, '-')}.har`,
        filters: [{ name: 'HAR', extensions: ['har'] }],
      })
      if (!path) return
      await api.exportHar(id, path)
      notify(t('sessions.exported', { path }))
    } catch (error) {
      notify(errorMessage(error), 'error')
    }
  }

  const confirm = async () => {
    if (!pending) return
    try {
      if (pending.kind === 'delete') await deleteSession(pending.id)
      else await clearSession(pending.id)
      resetTimeline()
      resetAnalysis()
    } catch (error) {
      notify(errorMessage(error), 'error')
    } finally {
      setPending(null)
    }
  }

  const groups = groupByDay(sessions)

  return (
    <div className={css.page}>
      <header className={css.pageHeader}>
        <h1 className={css.pageTitle}>{t('sessions.title')}</h1>
        <div className={css.pageSpacer} />
        <Button icon="upload" onClick={importHar}>
          {t('sessions.importHar')}
        </Button>
      </header>

      <div className={css.pageBody}>
        <div className={local.createRow}>
          <Input
            value={name}
            placeholder={t('sessions.namePlaceholder')}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') void create()
            }}
          />
          <Button variant="primary" icon="plus" onClick={create}>
            {t('sessions.new')}
          </Button>
        </div>

        {sessions.length === 0 ? (
          <Empty title={t('sessions.empty')} hint={t('sessions.emptyHint')} />
        ) : (
          groups.map(([label, group]) => (
            <section key={label} className={local.group}>
              <h2 className={css.sectionTitle}>{label}</h2>
              {group.map((s) => (
                <div
                  key={s.id}
                  className={`${local.card} ${session?.id === s.id ? local.cardActive : ''}`}
                >
                  <div className={local.cardMain} onClick={() => void open(s.id)}>
                    <div className={local.cardTitleRow}>
                      <StatusDot live={s.status === 'capturing'} />
                      {renaming === s.id ? (
                        <Input
                          autoFocus
                          value={renameValue}
                          onClick={(e) => e.stopPropagation()}
                          onChange={(e) => setRenameValue(e.target.value)}
                          onBlur={() => {
                            void renameSession(s.id, renameValue)
                            setRenaming(null)
                          }}
                          onKeyDown={(e) => {
                            if (e.key === 'Enter') e.currentTarget.blur()
                            if (e.key === 'Escape') setRenaming(null)
                          }}
                        />
                      ) : (
                        <span className={local.cardName}>{s.name}</span>
                      )}
                      <Badge>{t(`state.${s.status}`)}</Badge>
                    </div>
                    <div className={local.cardMeta}>
                      <span>{dateLabel(s.created_at)}</span>
                      <span>{t('sessions.requests', { n: count(s.request_count) })}</span>
                      {s.ignored_count > 0 ? (
                        <span>
                          {t('capture.ignored')} {count(s.ignored_count)}
                        </span>
                      ) : null}
                      {s.domains.length > 0 ? (
                        <span className={local.domains}>{s.domains.join(' · ')}</span>
                      ) : null}
                    </div>
                  </div>
                  <div className={local.cardActions}>
                    <IconButton
                      icon="download"
                      label={t('sessions.exportHar')}
                      onClick={() => void exportHar(s.id, s.name)}
                    />
                    <IconButton
                      icon="capture"
                      label={t('sessions.rename')}
                      onClick={() => {
                        setRenaming(s.id)
                        setRenameValue(s.name)
                      }}
                    />
                    <IconButton
                      icon="refresh"
                      label={t('sessions.clear')}
                      onClick={() => setPending({ kind: 'clear', id: s.id })}
                    />
                    <IconButton
                      icon="trash"
                      label={t('sessions.delete')}
                      onClick={() => setPending({ kind: 'delete', id: s.id })}
                    />
                  </div>
                </div>
              ))}
            </section>
          ))
        )}
      </div>

      <ConfirmDialog
        open={pending !== null}
        title={pending?.kind === 'delete' ? t('sessions.delete') : t('sessions.clear')}
        message={
          pending?.kind === 'delete' ? t('sessions.confirmDelete') : t('sessions.confirmClear')
        }
        destructive
        onConfirm={confirm}
        onCancel={() => setPending(null)}
      />
    </div>
  )
}

function groupByDay<T extends { created_at: number }>(items: T[]): [string, T[]][] {
  const map = new Map<string, T[]>()
  for (const item of items) {
    const key = relativeDay(item.created_at)
    map.set(key, [...(map.get(key) ?? []), item])
  }
  return [...map.entries()]
}
