import { Chip } from '@/components/ui'
import { useApp } from '@/stores/app'
import { useCapture } from '@/stores/capture'
import { useTimeline } from '@/stores/timeline'

import css from './timeline.module.css'

const METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE']
const STATUS_CLASSES = [2, 3, 4, 5]
const FAMILIES = ['json', 'html', 'script', 'style', 'image', 'font', 'media', 'other']
const IMPORTANCE = ['high', 'medium', 'low'] as const

export function FilterBar() {
  const t = useApp((s) => s.t)
  const filters = useTimeline((s) => s.filters)
  const toggleIn = useTimeline((s) => s.toggleIn)
  const toggleStatusClass = useTimeline((s) => s.toggleStatusClass)
  const setFilter = useTimeline((s) => s.setFilter)
  const hosts = useCapture((s) => s.hosts)

  const quick = [
    { key: 'only_api', label: t('filter.onlyApi') },
    { key: 'only_errors', label: t('filter.onlyErrors') },
    { key: 'only_json', label: t('filter.onlyJson') },
    { key: 'only_with_cookies', label: t('filter.onlyCookies') },
    { key: 'only_with_body', label: t('filter.onlyBody') },
    { key: 'only_authenticated', label: t('filter.onlyAuth') },
  ] as const

  return (
    <div className={css.filterBar}>
      <div className={css.filterGroup}>
        <span className={css.filterLabel}>{t('filter.method')}</span>
        {METHODS.map((m) => (
          <Chip key={m} active={filters.methods.includes(m)} onClick={() => toggleIn('methods', m)}>
            {m}
          </Chip>
        ))}
      </div>

      <span className={css.filterDivider} />

      <div className={css.filterGroup}>
        <span className={css.filterLabel}>{t('filter.status')}</span>
        {STATUS_CLASSES.map((c) => (
          <Chip
            key={c}
            active={filters.status_classes.includes(c)}
            onClick={() => toggleStatusClass(c)}
          >
            {c}xx
          </Chip>
        ))}
      </div>

      <span className={css.filterDivider} />

      <div className={css.filterGroup}>
        <span className={css.filterLabel}>{t('filter.type')}</span>
        {FAMILIES.map((f) => (
          <Chip
            key={f}
            active={filters.families.includes(f)}
            onClick={() => toggleIn('families', f)}
          >
            {f}
          </Chip>
        ))}
      </div>

      <span className={css.filterDivider} />

      <div className={css.filterGroup}>
        <span className={css.filterLabel}>{t('filter.importance')}</span>
        {IMPORTANCE.map((i) => (
          <Chip
            key={i}
            active={filters.importance.includes(i)}
            onClick={() => toggleIn('importance', i)}
          >
            {t(`importance.${i}`)}
          </Chip>
        ))}
      </div>

      {hosts.length > 1 ? (
        <>
          <span className={css.filterDivider} />
          <div className={css.filterGroup}>
            <span className={css.filterLabel}>{t('filter.host')}</span>
            {hosts.slice(0, 8).map((h) => (
              <Chip
                key={h.host}
                active={filters.hosts.includes(h.host)}
                onClick={() => toggleIn('hosts', h.host)}
                title={`${h.host} · ${h.count}`}
              >
                {h.host}
              </Chip>
            ))}
          </div>
        </>
      ) : null}

      <span className={css.filterDivider} />

      <div className={css.filterGroup}>
        <span className={css.filterLabel}>{t('filter.quick')}</span>
        {quick.map((q) => (
          <Chip
            key={q.key}
            active={filters[q.key]}
            onClick={() => setFilter(q.key, !filters[q.key])}
          >
            {q.label}
          </Chip>
        ))}
      </div>
    </div>
  )
}
