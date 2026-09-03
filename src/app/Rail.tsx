import { Icon, type IconName } from '@/components/ui'
import type { TranslationKey } from '@/i18n'
import { useApp, type Route } from '@/stores/app'

import css from './shell.module.css'

const items: { route: Route; icon: IconName; label: TranslationKey }[] = [
  { route: 'capture', icon: 'capture', label: 'nav.capture' },
  { route: 'repeater', icon: 'repeater', label: 'nav.repeater' },
  { route: 'analysis', icon: 'analysis', label: 'nav.analysis' },
  { route: 'flow', icon: 'flow', label: 'nav.flow' },
  { route: 'sessions', icon: 'sessions', label: 'nav.sessions' },
]

const footerItems: { route: Route; icon: IconName; label: TranslationKey }[] = [
  { route: 'ca', icon: 'certificate', label: 'nav.certificate' },
  { route: 'settings', icon: 'settings', label: 'nav.settings' },
]

export function Rail() {
  const route = useApp((s) => s.route)
  const setRoute = useApp((s) => s.setRoute)
  const t = useApp((s) => s.t)
  const theme = useApp((s) => s.theme)
  const setTheme = useApp((s) => s.setTheme)

  const button = (item: (typeof items)[number]) => (
    <button
      key={item.route}
      type="button"
      className={`${css.railButton} ${route === item.route ? css.railButtonActive : ''}`}
      title={t(item.label)}
      aria-label={t(item.label)}
      aria-current={route === item.route}
      onClick={() => setRoute(item.route)}
    >
      <Icon name={item.icon} size={17} />
    </button>
  )

  return (
    <nav className={css.rail} aria-label={t('app.name')}>
      <div className={css.mark} aria-hidden="true">
        V
      </div>
      {items.map(button)}
      <div className={css.railSpacer} />
      <button
        type="button"
        className={css.railButton}
        title={t('settings.theme')}
        aria-label={t('settings.theme')}
        onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
      >
        <Icon name={theme === 'dark' ? 'sun' : 'moon'} size={16} />
      </button>
      {footerItems.map(button)}
    </nav>
  )
}
