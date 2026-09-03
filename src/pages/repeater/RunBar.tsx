import { Button, Icon, Input, Select, Toggle } from '@/components/ui'
import { useApp } from '@/stores/app'
import { useRepeater } from '@/stores/repeater'
import type { ReplayMode } from '@/types/repeater'

import css from './repeater.module.css'

export function RunBar() {
  const t = useApp((s) => s.t)
  const { options, setOptions, run, running, progress, draft, persistDraft } = useRepeater()

  if (!draft) return null

  const start = async () => {
    await persistDraft()
    await run()
  }

  const pct = progress && progress.total > 0 ? (progress.completed / progress.total) * 100 : 0

  return (
    <>
      {options.mode === 'concurrent' ? (
        <div className={css.warnNote} style={{ margin: 'var(--space-4) var(--space-5) 0' }}>
          <Icon name="warning" size={12} />
          <span>{t('repeater.concurrentWarning')}</span>
        </div>
      ) : null}

      <div className={css.runBar}>
        <label className={css.runField}>
          {t('repeater.iterations')}
          <Input
            className={css.numberInput}
            type="number"
            min={1}
            max={500}
            value={options.iterations}
            onChange={(e) =>
              setOptions({ iterations: Math.max(1, Math.min(500, Number(e.target.value) || 1)) })
            }
          />
        </label>

        <label className={css.runField}>
          {t('repeater.mode')}
          <Select
            style={{ width: 118 }}
            value={options.mode}
            onChange={(e) => setOptions({ mode: e.target.value as ReplayMode })}
          >
            <option value="sequential">{t('repeater.sequential')}</option>
            <option value="concurrent">{t('repeater.concurrent')}</option>
          </Select>
        </label>

        <label className={css.runField}>
          {t('repeater.delay')}
          <Select
            style={{ width: 92 }}
            value={String(options.delay_ms)}
            onChange={(e) => setOptions({ delay_ms: Number(e.target.value) })}
            disabled={options.mode === 'concurrent'}
          >
            {[0, 100, 250, 500, 1000, 2000, 5000].map((ms) => (
              <option key={ms} value={ms}>
                {ms === 0 ? '0 ms' : ms < 1000 ? `${ms} ms` : `${ms / 1000} s`}
              </option>
            ))}
          </Select>
        </label>

        <Toggle
          checked={options.follow_redirects}
          onChange={(v) => setOptions({ follow_redirects: v })}
          label={t('repeater.followRedirects')}
        />

        {running && progress ? (
          <div className={css.progress} title={`${progress.completed}/${progress.total}`}>
            <div className={css.progressFill} style={{ width: `${pct}%` }} />
          </div>
        ) : (
          <div className={css.spacer} />
        )}

        <Button variant="primary" icon="play" onClick={start} loading={running}>
          {options.iterations > 1
            ? t('repeater.sendN', { n: options.iterations })
            : t('repeater.send')}
        </Button>
      </div>
    </>
  )
}
