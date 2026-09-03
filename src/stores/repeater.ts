import { create } from 'zustand'

import { api, errorMessage } from '@/lib/ipc'
import type { RepeaterDraft, ReplayOptions, ReplayResult } from '@/types/repeater'

import { useApp } from './app'

export const defaultOptions: ReplayOptions = {
  iterations: 1,
  mode: 'sequential',
  delay_ms: 0,
  follow_redirects: false,
  timeout_ms: 30_000,
}

interface RepeaterStore {
  drafts: RepeaterDraft[]
  activeId: string | null
  draft: RepeaterDraft | null
  options: ReplayOptions
  results: ReplayResult[]
  running: boolean
  progress: { completed: number; total: number } | null
  dirty: boolean

  loadDrafts: (sessionId: string) => Promise<void>
  selectDraft: (draftId: string | null) => Promise<void>
  createDraft: (sessionId: string) => Promise<void>
  adoptDraft: (draft: RepeaterDraft) => Promise<void>
  patchDraft: (patch: Partial<RepeaterDraft>) => void
  persistDraft: () => Promise<void>
  removeDraft: (draftId: string) => Promise<void>
  setOptions: (patch: Partial<ReplayOptions>) => void
  run: () => Promise<void>
  pushResult: (result: ReplayResult) => void
  setProgress: (progress: { completed: number; total: number } | null) => void
  clearHistory: () => Promise<void>
}

export const useRepeater = create<RepeaterStore>((set, get) => ({
  drafts: [],
  activeId: null,
  draft: null,
  options: { ...defaultOptions },
  results: [],
  running: false,
  progress: null,
  dirty: false,

  loadDrafts: async (sessionId) => {
    try {
      const drafts = await api.listDrafts(sessionId)
      set({ drafts })
      if (!get().activeId && drafts.length > 0) {
        await get().selectDraft(drafts[0].id)
      }
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  selectDraft: async (draftId) => {
    if (!draftId) {
      set({ activeId: null, draft: null, results: [] })
      return
    }
    try {
      const draft = await api.getDraft(draftId)
      const results = await api.listReplays(draftId, 200)
      set({ activeId: draftId, draft, results, dirty: false, progress: null })
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  createDraft: async (sessionId) => {
    try {
      const draft = await api.newDraft(sessionId)
      set((s) => ({ drafts: [draft, ...s.drafts], activeId: draft.id, draft, results: [] }))
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  adoptDraft: async (draft) => {
    set((s) => ({
      drafts: [draft, ...s.drafts.filter((d) => d.id !== draft.id)],
      activeId: draft.id,
      draft,
      results: [],
      dirty: false,
    }))
  },

  patchDraft: (patch) =>
    set((s) => (s.draft ? { draft: { ...s.draft, ...patch }, dirty: true } : s)),

  persistDraft: async () => {
    const draft = get().draft
    if (!draft || !get().dirty) return
    try {
      const saved = await api.saveDraft(draft)
      set((s) => ({
        draft: saved,
        dirty: false,
        drafts: s.drafts.map((d) => (d.id === saved.id ? saved : d)),
      }))
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  removeDraft: async (draftId) => {
    try {
      await api.deleteDraft(draftId)
      set((s) => {
        const drafts = s.drafts.filter((d) => d.id !== draftId)
        const isActive = s.activeId === draftId
        return {
          drafts,
          activeId: isActive ? null : s.activeId,
          draft: isActive ? null : s.draft,
          results: isActive ? [] : s.results,
        }
      })
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  setOptions: (patch) => set((s) => ({ options: { ...s.options, ...patch } })),

  run: async () => {
    const { draft, options } = get()
    if (!draft || get().running) return
    set({ running: true, progress: { completed: 0, total: options.iterations } })
    try {
      await api.runReplay(draft, options)
      set({ dirty: false })
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    } finally {
      set({ running: false, progress: null })
    }
  },

  pushResult: (result) =>
    set((s) =>
      s.activeId === result.draft_id ? { results: [result, ...s.results].slice(0, 400) } : s,
    ),

  setProgress: (progress) => set({ progress }),

  clearHistory: async () => {
    const id = get().activeId
    if (!id) return
    await api.clearReplays(id)
    set({ results: [] })
  },
}))
