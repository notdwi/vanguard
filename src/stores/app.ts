import { create } from 'zustand'

import type { Language } from '@/i18n'
import { translate, type TranslationKey, type Vars } from '@/i18n'
import { api } from '@/lib/ipc'
import type { AppSettings } from '@/types/analysis'

export type Theme = 'light' | 'dark' | 'system'
export type Route = 'capture' | 'repeater' | 'analysis' | 'flow' | 'sessions' | 'ca' | 'settings'

interface Toast {
  id: number
  message: string
  tone: 'info' | 'error'
}

interface AppStore {
  route: Route
  settings: AppSettings
  theme: Theme
  toasts: Toast[]
  ready: boolean
  setRoute: (route: Route) => void
  setTheme: (theme: Theme) => void
  loadSettings: () => Promise<void>
  saveSettings: (patch: Partial<AppSettings>) => Promise<void>
  notify: (message: string, tone?: 'info' | 'error') => void
  dismiss: (id: number) => void
  t: (key: TranslationKey, vars?: Vars) => string
}

const defaultSettings: AppSettings = {
  language: 'en',
  proxy_port: 8080,
  mask_secrets: true,
  timeline_page_size: 500,
  auto_analyse: true,
  default_replay_delay_ms: 0,
  sensitive_headers: [],
}

const THEME_KEY = 'vanguard.theme'

function readTheme(): Theme {
  try {
    const stored = localStorage.getItem(THEME_KEY)
    if (stored === 'light' || stored === 'dark' || stored === 'system') return stored
  } catch {
    /* storage may be unavailable */
  }
  return 'system'
}

export function applyTheme(theme: Theme) {
  const root = document.documentElement
  if (theme === 'system') {
    root.removeAttribute('data-theme')
  } else {
    root.setAttribute('data-theme', theme)
  }
  try {
    localStorage.setItem(THEME_KEY, theme)
  } catch {
    /* ignore */
  }
}

let toastId = 0

export const useApp = create<AppStore>((set, get) => ({
  route: 'capture',
  settings: defaultSettings,
  theme: readTheme(),
  toasts: [],
  ready: false,

  setRoute: (route) => set({ route }),

  setTheme: (theme) => {
    applyTheme(theme)
    set({ theme })
  },

  loadSettings: async () => {
    try {
      const settings = await api.getSettings()
      set({ settings, ready: true })
    } catch {
      set({ ready: true })
    }
  },

  saveSettings: async (patch) => {
    const next = { ...get().settings, ...patch }
    set({ settings: next })
    try {
      await api.saveSettings(next)
    } catch (error) {
      get().notify(String(error), 'error')
    }
  },

  notify: (message, tone = 'info') => {
    const id = ++toastId
    set((s) => ({ toasts: [...s.toasts, { id, message, tone }] }))
    setTimeout(() => get().dismiss(id), tone === 'error' ? 8000 : 3500)
  },

  dismiss: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),

  t: (key, vars) => translate(get().settings.language as Language, key, vars),
}))
