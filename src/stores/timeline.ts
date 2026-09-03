import { create } from 'zustand'

import { api, errorMessage } from '@/lib/ipc'
import type { RequestDetail, TimelineQuery, TimelineRow } from '@/types/core'

import { useApp } from './app'

export const emptyQuery: Omit<TimelineQuery, 'session_id'> = {
  search: null,
  methods: [],
  status_classes: [],
  families: [],
  hosts: [],
  importance: [],
  only_api: false,
  only_errors: false,
  only_json: false,
  only_with_cookies: false,
  only_with_body: false,
  only_authenticated: false,
  search_bodies: false,
  limit: 500,
  offset: 0,
}

interface TimelineStore {
  rows: TimelineRow[]
  total: number
  filters: Omit<TimelineQuery, 'session_id'>
  selectedId: string | null
  detail: RequestDetail | null
  loading: boolean
  live: boolean

  setFilter: <K extends keyof Omit<TimelineQuery, 'session_id'>>(
    key: K,
    value: Omit<TimelineQuery, 'session_id'>[K],
  ) => void
  toggleIn: (key: 'methods' | 'families' | 'hosts' | 'importance', value: string) => void
  toggleStatusClass: (value: number) => void
  clearFilters: () => void
  activeFilterCount: () => number

  load: (sessionId: string) => Promise<void>
  select: (requestId: string | null) => Promise<void>
  applyLiveRow: (row: TimelineRow) => void
  patchRow: (id: string, patch: Partial<TimelineRow>) => void
  setLive: (live: boolean) => void
  reset: () => void
}

export const useTimeline = create<TimelineStore>((set, get) => ({
  rows: [],
  total: 0,
  filters: { ...emptyQuery },
  selectedId: null,
  detail: null,
  loading: false,
  live: true,

  setFilter: (key, value) =>
    set((s) => ({ filters: { ...s.filters, [key]: value, offset: 0 } })),

  toggleIn: (key, value) =>
    set((s) => {
      const current = s.filters[key]
      const next = current.includes(value)
        ? current.filter((v) => v !== value)
        : [...current, value]
      return { filters: { ...s.filters, [key]: next, offset: 0 } }
    }),

  toggleStatusClass: (value) =>
    set((s) => {
      const current = s.filters.status_classes
      const next = current.includes(value)
        ? current.filter((v) => v !== value)
        : [...current, value]
      return { filters: { ...s.filters, status_classes: next, offset: 0 } }
    }),

  clearFilters: () => set((s) => ({ filters: { ...emptyQuery, limit: s.filters.limit } })),

  activeFilterCount: () => {
    const f = get().filters
    let n = 0
    if (f.search) n += 1
    n += f.methods.length + f.families.length + f.hosts.length + f.importance.length
    n += f.status_classes.length
    for (const key of [
      'only_api',
      'only_errors',
      'only_json',
      'only_with_cookies',
      'only_with_body',
      'only_authenticated',
    ] as const) {
      if (f[key]) n += 1
    }
    return n
  },

  load: async (sessionId) => {
    set({ loading: true })
    try {
      const page = await api.timeline({ session_id: sessionId, ...get().filters })
      set({ rows: page.rows, total: page.total })
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    } finally {
      set({ loading: false })
    }
  },

  select: async (requestId) => {
    set({ selectedId: requestId })
    if (!requestId) {
      set({ detail: null })
      return
    }
    try {
      set({ detail: await api.requestDetail(requestId) })
    } catch (error) {
      set({ detail: null })
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  applyLiveRow: (row) =>
    set((s) => {
      if (!s.live) return s
      if (s.rows.some((r) => r.id === row.id)) return s
      return { rows: [...s.rows, row], total: s.total + 1 }
    }),

  patchRow: (id, patch) =>
    set((s) => ({
      rows: s.rows.map((r) => (r.id === id ? { ...r, ...patch } : r)),
    })),

  setLive: (live) => set({ live }),

  reset: () => set({ rows: [], total: 0, selectedId: null, detail: null }),
}))
