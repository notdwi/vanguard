import { useEffect } from 'react'

import { IconButton } from '@/components/ui'
import { useBackendEvents, useCounterSync } from '@/hooks/useBackendEvents'
import { AnalysisPage } from '@/pages/AnalysisPage'
import { CaPage } from '@/pages/CaPage'
import { CapturePage } from '@/pages/CapturePage'
import { FlowPage } from '@/pages/FlowPage'
import { RepeaterPage } from '@/pages/RepeaterPage'
import { SessionsPage } from '@/pages/SessionsPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { applyTheme, useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'

import { Rail } from './Rail'
import { Topbar } from './Topbar'
import css from './shell.module.css'

const pages = {
  capture: CapturePage,
  repeater: RepeaterPage,
  analysis: AnalysisPage,
  flow: FlowPage,
  sessions: SessionsPage,
  ca: CaPage,
  settings: SettingsPage,
}

export function App() {
  const route = useApp((s) => s.route)
  const theme = useApp((s) => s.theme)
  const toasts = useApp((s) => s.toasts)
  const dismiss = useApp((s) => s.dismiss)
  const loadSettings = useApp((s) => s.loadSettings)
  const status = useCapture((s) => s.status)
  const refreshStatus = useCapture((s) => s.refreshStatus)
  const refreshSessions = useCapture((s) => s.refreshSessions)

  useBackendEvents()
  useCounterSync(status.state === 'capturing')

  useEffect(() => {
    applyTheme(theme)
  }, [theme])

  useEffect(() => {
    void loadSettings()
    void refreshStatus()
    void refreshSessions()
  }, [loadSettings, refreshStatus, refreshSessions])

  const Page = pages[route]

  return (
    <div className={css.shell}>
      <Rail />
      <div className={css.main}>
        <Topbar />
        <div className={css.content}>
          <Page />
        </div>
      </div>

      {toasts.length > 0 ? (
        <div className={css.toasts}>
          {toasts.map((toast) => (
            <div
              key={toast.id}
              className={`${css.toast} ${toast.tone === 'error' ? css.toastError : ''}`}
              role="status"
            >
              <span className={css.toastMessage}>{toast.message}</span>
              <IconButton
                icon="close"
                size={12}
                label="Dismiss"
                onClick={() => dismiss(toast.id)}
              />
            </div>
          ))}
        </div>
      ) : null}
    </div>
  )
}
