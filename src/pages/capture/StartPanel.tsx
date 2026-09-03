import { useEffect, useState } from 'react'

import { Badge, Button, Icon, IconButton, Select } from '@/components/ui'
import { api, errorMessage } from '@/lib/ipc'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import type { BrowserOption, CaInfo } from '@/types/analysis'

import css from './capture.module.css'

export function StartPanel({
  onToggleScope,
  scopeOpen,
}: {
  onToggleScope: () => void
  scopeOpen: boolean
}) {
  const t = useApp((s) => s.t)
  const notify = useApp((s) => s.notify)
  const setRoute = useApp((s) => s.setRoute)
  const status = useCapture((s) => s.status)
  const session = useCapture((s) => s.session)

  const [browsers, setBrowsers] = useState<BrowserOption[]>([])
  const [browserId, setBrowserId] = useState('')
  const [ca, setCa] = useState<CaInfo | null>(null)

  useEffect(() => {
    void api.listBrowsers().then((list) => {
      setBrowsers(list)
      if (list.length > 0) setBrowserId((current) => current || list[0].id)
    })
    void api
      .caInfo()
      .then(setCa)
      .catch(() => undefined)
  }, [status.state])

  const selected = browsers.find((b) => b.id === browserId)

  const launch = async () => {
    try {
      await api.launchBrowser(browserId, 'about:blank')
    } catch (error) {
      notify(errorMessage(error), 'error')
    }
  }

  const running = status.state === 'capturing' || status.state === 'paused'
  const scopeLabel = session ? t(`scope.${camel(session.config.mode)}`) : t('scope.allTraffic')

  const hint = selected
    ? selected.kind === 'firefox'
      ? selected.uses_system_trust
        ? t('capture.firefoxHint')
        : t('capture.firefoxLinuxHint')
      : t('capture.browserHint')
    : t('capture.browserHint')

  return (
    <div className={css.bar}>
      <span className={css.label}>{t('capture.scope')}</span>
      <Button small icon="filter" onClick={onToggleScope} aria-expanded={scopeOpen}>
        {scopeLabel}
      </Button>
      {session && session.config.include_domains.length > 0 ? (
        <Badge>{session.config.include_domains.slice(0, 2).join(', ')}</Badge>
      ) : null}

      <div className={css.spacer} />

      {ca && !ca.installed ? (
        <button type="button" className={css.warn} onClick={() => setRoute('ca')}>
          <Icon name="warning" size={13} />
          {t('capture.needCa')}
        </button>
      ) : null}

      {browsers.length > 0 ? (
        <>
          <Select
            value={browserId}
            onChange={(e) => setBrowserId(e.target.value)}
            style={{ width: 158 }}
          >
            {browsers.map((b) => (
              <option key={b.id} value={b.id}>
                {b.name}
              </option>
            ))}
          </Select>
          <Button small icon="browser" onClick={launch} disabled={!running}>
            {t('capture.openBrowser')}
          </Button>
          <IconButton icon="warning" size={13} label={hint} />
        </>
      ) : (
        <span className={css.warn}>{t('capture.noBrowsers')}</span>
      )}
    </div>
  )
}

function camel(mode: string): 'allTraffic' | 'exactHost' | 'domainAndSubdomains' {
  if (mode === 'exact-host') return 'exactHost'
  if (mode === 'domain-and-subdomains') return 'domainAndSubdomains'
  return 'allTraffic'
}
