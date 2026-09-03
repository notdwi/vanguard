import { Badge, Button, Icon, IconButton, StatusDot } from '@/components/ui'
import { count } from '@/lib/format'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'

import css from './shell.module.css'

export function Topbar() {
  const t = useApp((s) => s.t)
  const setRoute = useApp((s) => s.setRoute)
  const status = useCapture((s) => s.status)
  const session = useCapture((s) => s.session)
  const busy = useCapture((s) => s.busy)
  const start = useCapture((s) => s.start)
  const stop = useCapture((s) => s.stop)
  const togglePause = useCapture((s) => s.togglePause)

  const live = status.state === 'capturing'
  const active = live || status.state === 'paused'

  return (
    <header className={css.topbar}>
      <button
        type="button"
        className={css.sessionPicker}
        onClick={() => setRoute('sessions')}
        title={t('nav.sessions')}
      >
        <Icon name="sessions" size={13} />
        <span className={css.sessionName}>
          {session?.name ?? status.session_name ?? t('sessions.new')}
        </span>
        <Icon name="chevronDown" size={12} />
      </button>

      <div className={`${css.statusPill} ${live ? css.statusPillLive : ''}`}>
        <StatusDot live={live} />
        {t(`state.${status.state}`)}
      </div>

      {status.proxy_addr ? (
        <span className={css.proxyTag}>
          {t('capture.proxy')} {status.proxy_addr}
        </span>
      ) : null}

      <div className={css.topbarSpacer} />

      {active ? (
        <div className={css.counters}>
          <span>
            <span className={css.counterLabel}>{t('capture.captured')}</span>
            {count(status.captured)}
          </span>
          <span>
            <span className={css.counterLabel}>{t('capture.ignored')}</span>
            {count(status.ignored)}
          </span>
        </div>
      ) : session ? (
        <Badge>{t('sessions.requests', { n: count(session.request_count) })}</Badge>
      ) : null}

      {active ? (
        <>
          <IconButton
            icon={status.state === 'paused' ? 'play' : 'pause'}
            label={status.state === 'paused' ? t('capture.resume') : t('capture.pause')}
            onClick={togglePause}
          />
          <Button icon="stop" onClick={stop} loading={busy}>
            {t('capture.stop')}
          </Button>
        </>
      ) : (
        <Button variant="primary" icon="play" onClick={start} loading={busy} disabled={!session}>
          {t('capture.start')}
        </Button>
      )}
    </header>
  )
}
