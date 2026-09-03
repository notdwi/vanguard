import { useEffect, useState } from 'react'

import { Button, Empty } from '@/components/ui'
import { Inspector } from '@/components/inspector/Inspector'
import { SplitPane } from '@/components/layout/SplitPane'
import { Timeline } from '@/components/timeline/Timeline'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import { useTimeline } from '@/stores/timeline'

import { ScopePanel } from './capture/ScopePanel'
import { StartPanel } from './capture/StartPanel'

export function CapturePage() {
  const t = useApp((s) => s.t)
  const session = useCapture((s) => s.session)
  const sessions = useCapture((s) => s.sessions)
  const createSession = useCapture((s) => s.createSession)
  const openSession = useCapture((s) => s.openSession)
  const reset = useTimeline((s) => s.reset)
  const [showScope, setShowScope] = useState(false)

  useEffect(() => {
    if (session) return
    if (sessions.length > 0) void openSession(sessions[0].id)
  }, [session, sessions, openSession])

  useEffect(() => {
    reset()
  }, [session?.id, reset])

  if (!session) {
    return (
      <Empty
        title={t('sessions.empty')}
        hint={t('sessions.emptyHint')}
        action={
          <Button variant="primary" icon="plus" onClick={() => void createSession('')}>
            {t('sessions.new')}
          </Button>
        }
      />
    )
  }

  return (
    <>
      <StartPanel onToggleScope={() => setShowScope((v) => !v)} scopeOpen={showScope} />
      {showScope ? <ScopePanel /> : null}
      <SplitPane
        storageKey="capture"
        left={<Timeline />}
        right={<Inspector />}
        initial={0.55}
      />
    </>
  )
}
