export type IconName =
  | 'capture'
  | 'repeater'
  | 'analysis'
  | 'flow'
  | 'sessions'
  | 'certificate'
  | 'settings'
  | 'play'
  | 'stop'
  | 'pause'
  | 'search'
  | 'filter'
  | 'copy'
  | 'check'
  | 'close'
  | 'plus'
  | 'trash'
  | 'chevronRight'
  | 'chevronDown'
  | 'external'
  | 'download'
  | 'upload'
  | 'refresh'
  | 'eye'
  | 'eyeOff'
  | 'arrowRight'
  | 'browser'
  | 'warning'
  | 'sun'
  | 'moon'

const paths: Record<IconName, string> = {
  capture: 'M3 12h4l3-7 4 14 3-7h4',
  repeater: 'M4 7h11a4 4 0 0 1 0 8H8m0 0 3-3m-3 3 3 3M4 7l3-3M4 7l3 3',
  analysis: 'M4 19V9m5 10V5m5 14v-7m5 7V8',
  flow: 'M12 4v4m0 8v4M6 12h4m4 0h4M12 8a2 2 0 1 0 0 4 2 2 0 0 0 0-4Zm0 8a2 2 0 1 0 0 4 2 2 0 0 0 0-4ZM4 10a2 2 0 1 0 0 4 2 2 0 0 0 0-4Zm16 0a2 2 0 1 0 0 4 2 2 0 0 0 0-4Z',
  sessions: 'M4 6h16M4 12h16M4 18h16',
  certificate: 'M12 3 4 6v5c0 4.4 3.2 8.4 8 10 4.8-1.6 8-5.6 8-10V6l-8-3Zm-3 9 2 2 4-4',
  settings:
    'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm8-3-1.8-.6-.5-1.2.9-1.7-1.6-1.6-1.7.9-1.2-.5L13.5 5h-2.3l-.6 1.8-1.2.5-1.7-.9L6 8l.9 1.7-.5 1.2L4.6 12l.6 2 1.8.6.5 1.2-.9 1.7 1.6 1.6 1.7-.9 1.2.5.6 1.8h2.3l.6-1.8 1.2-.5 1.7.9 1.6-1.6-.9-1.7.5-1.2 1.8-.6',
  play: 'M7 4.5v15l13-7.5-13-7.5Z',
  stop: 'M6 6h12v12H6z',
  pause: 'M9 5v14M15 5v14',
  search: 'M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14Zm5.5-1.5L21 21',
  filter: 'M4 6h16l-6.5 7.5V20l-3-2v-4.5L4 6Z',
  copy: 'M9 9h9a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1v-9a1 1 0 0 1 1-1Zm-3 6H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1v1',
  check: 'M4 12.5 9 17.5 20 6.5',
  close: 'M6 6l12 12M18 6 6 18',
  plus: 'M12 5v14M5 12h14',
  trash: 'M4 7h16M9 7V4h6v3m-8 0 1 13h8l1-13M10 11v6m4-6v6',
  chevronRight: 'M9 5l7 7-7 7',
  chevronDown: 'M5 9l7 7 7-7',
  external: 'M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5',
  download: 'M12 4v11m0 0 4-4m-4 4-4-4M4 19h16',
  upload: 'M12 20V9m0 0 4 4m-4-4-4 4M4 5h16',
  refresh: 'M4 12a8 8 0 0 1 13.7-5.7L20 8M20 4v4h-4M20 12a8 8 0 0 1-13.7 5.7L4 16m0 4v-4h4',
  eye: 'M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12Zm9.5 2.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z',
  eyeOff: 'M4 4l16 16M9.9 5.9A9.6 9.6 0 0 1 12 5.5c6 0 9.5 6.5 9.5 6.5a17 17 0 0 1-3.3 4M6.3 8.2A17 17 0 0 0 2.5 12S6 18.5 12 18.5c1 0 1.9-.2 2.7-.5',
  arrowRight: 'M4 12h15m0 0-5-5m5 5-5 5',
  browser: 'M3 5h18v14H3V5Zm0 4h18M6.5 7h.01M9 7h.01',
  warning: 'M12 4 2.5 20h19L12 4Zm0 6v5m0 3h.01',
  sun: 'M12 7.5a4.5 4.5 0 1 0 0 9 4.5 4.5 0 0 0 0-9ZM12 2v2m0 16v2M2 12h2m16 0h2M4.9 4.9l1.5 1.5m11.2 11.2 1.5 1.5M19.1 4.9l-1.5 1.5M6.4 17.6l-1.5 1.5',
  moon: 'M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5Z',
}

interface IconProps {
  name: IconName
  size?: number
  strokeWidth?: number
  filled?: boolean
}

export function Icon({ name, size = 16, strokeWidth = 1.6, filled = false }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill={filled ? 'currentColor' : 'none'}
      stroke={filled ? 'none' : 'currentColor'}
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      <path d={paths[name]} />
    </svg>
  )
}
