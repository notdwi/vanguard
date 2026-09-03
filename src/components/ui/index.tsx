import type { ButtonHTMLAttributes, InputHTMLAttributes, ReactNode } from 'react'

import { Icon, type IconName } from './Icon'
import fb from './feedback.module.css'
import css from './ui.module.css'

type ButtonVariant = 'default' | 'primary' | 'ghost' | 'danger'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  small?: boolean
  icon?: IconName
  loading?: boolean
}

export function Button({
  variant = 'default',
  small,
  icon,
  loading,
  children,
  className,
  ...rest
}: ButtonProps) {
  const classes = [
    css.button,
    variant === 'primary' && css.primary,
    variant === 'ghost' && css.ghost,
    variant === 'danger' && css.danger,
    small && css.small,
    className,
  ]
    .filter(Boolean)
    .join(' ')

  return (
    <button type="button" className={classes} {...rest}>
      {loading ? <span className={fb.spinner} /> : icon ? <Icon name={icon} size={13} /> : null}
      {children}
    </button>
  )
}

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  icon: IconName
  label: string
  active?: boolean
  size?: number
}

export function IconButton({ icon, label, active, size = 14, ...rest }: IconButtonProps) {
  return (
    <button
      type="button"
      className={`${css.iconButton} ${active ? css.iconButtonActive : ''}`}
      title={label}
      aria-label={label}
      {...rest}
    >
      <Icon name={icon} size={size} />
    </button>
  )
}

export function Input({
  mono,
  className,
  ...rest
}: InputHTMLAttributes<HTMLInputElement> & { mono?: boolean }) {
  return (
    <input
      className={[css.input, mono && css.mono, className].filter(Boolean).join(' ')}
      spellCheck={false}
      autoComplete="off"
      {...rest}
    />
  )
}

export function Textarea({
  className,
  ...rest
}: React.TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return <textarea className={[css.textarea, className].filter(Boolean).join(' ')} spellCheck={false} {...rest} />
}

export function Select({
  className,
  children,
  ...rest
}: React.SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select className={[css.select, className].filter(Boolean).join(' ')} {...rest}>
      {children}
    </select>
  )
}

export function Field({
  label,
  hint,
  children,
}: {
  label?: string
  hint?: string
  children: ReactNode
}) {
  return (
    <div className={css.field}>
      {label ? <span className={css.label}>{label}</span> : null}
      {children}
      {hint ? <span className={css.hint}>{hint}</span> : null}
    </div>
  )
}

export function Badge({
  children,
  tone = 'default',
  title,
}: {
  children: ReactNode
  tone?: 'default' | 'solid' | 'outline'
  title?: string
}) {
  const classes = [
    css.badge,
    tone === 'solid' && css.badgeSolid,
    tone === 'outline' && css.badgeOutline,
  ]
    .filter(Boolean)
    .join(' ')
  return (
    <span className={classes} title={title}>
      {children}
    </span>
  )
}

export function Toggle({
  checked,
  onChange,
  label,
  disabled,
}: {
  checked: boolean
  onChange: (value: boolean) => void
  label?: ReactNode
  disabled?: boolean
}) {
  return (
    <label className={css.toggle} style={disabled ? { opacity: 0.5 } : undefined}>
      <input
        type="checkbox"
        className={css.checkInput}
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className={`${css.toggleTrack} ${checked ? css.toggleTrackOn : ''}`}>
        <span className={css.toggleThumb} />
      </span>
      {label ? <span>{label}</span> : null}
    </label>
  )
}

export function Chip({
  active,
  onClick,
  children,
  title,
}: {
  active?: boolean
  onClick?: () => void
  children: ReactNode
  title?: string
}) {
  return (
    <button
      type="button"
      className={`${css.chip} ${active ? css.chipActive : ''}`}
      onClick={onClick}
      title={title}
    >
      {children}
    </button>
  )
}

export function Empty({
  title,
  hint,
  action,
}: {
  title: string
  hint?: string
  action?: ReactNode
}) {
  return (
    <div className={fb.empty}>
      <p className={fb.emptyTitle}>{title}</p>
      {hint ? <p className={fb.emptyHint}>{hint}</p> : null}
      {action ? <div className={fb.emptyAction}>{action}</div> : null}
    </div>
  )
}

export interface TabItem {
  id: string
  label: string
  count?: number
}

export function Tabs({
  items,
  active,
  onSelect,
}: {
  items: TabItem[]
  active: string
  onSelect: (id: string) => void
}) {
  return (
    <div className={fb.tabs} role="tablist">
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          role="tab"
          aria-selected={active === item.id}
          className={`${fb.tab} ${active === item.id ? fb.tabActive : ''}`}
          onClick={() => onSelect(item.id)}
        >
          {item.label}
          {item.count != null && item.count > 0 ? (
            <span className={fb.tabCount}>{item.count}</span>
          ) : null}
        </button>
      ))}
    </div>
  )
}

export function StatusDot({ live, ring }: { live?: boolean; ring?: boolean }) {
  return (
    <span
      className={[fb.statusDot, live && fb.statusDotLive, ring && fb.statusDotRing]
        .filter(Boolean)
        .join(' ')}
    />
  )
}

export function Spinner() {
  return <span className={fb.spinner} />
}

export function KeyValue({ items }: { items: { key: string; value: ReactNode }[] }) {
  return (
    <dl className={fb.kv}>
      {items.map((item) => (
        <div key={item.key} style={{ display: 'contents' }}>
          <dt className={fb.kvKey}>{item.key}</dt>
          <dd className={fb.kvValue} style={{ margin: 0 }}>
            {item.value}
          </dd>
        </div>
      ))}
    </dl>
  )
}

export { Icon }
export type { IconName }
