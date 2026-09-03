import { create } from 'zustand'

import { api, errorMessage } from '@/lib/ipc'
import type { AnalysisBundle, RequestAnalysis } from '@/types/analysis'

import { useApp } from './app'

interface AnalysisStore {
  bundle: AnalysisBundle | null
  requestAnalysis: Record<string, RequestAnalysis>
  loading: boolean
  run: (sessionId: string, refresh?: boolean) => Promise<void>
  forRequest: (sessionId: string, requestId: string) => Promise<void>
  reset: () => void
}

export const useAnalysis = create<AnalysisStore>((set, get) => ({
  bundle: null,
  requestAnalysis: {},
  loading: false,

  run: async (sessionId, refresh = false) => {
    set({ loading: true })
    try {
      const bundle = await api.analyseSession(sessionId, refresh)
      set({ bundle, requestAnalysis: {} })
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    } finally {
      set({ loading: false })
    }
  },

  forRequest: async (sessionId, requestId) => {
    if (get().requestAnalysis[requestId]) return
    try {
      const analysis = await api.analyseRequest(sessionId, requestId)
      set((s) => ({ requestAnalysis: { ...s.requestAnalysis, [requestId]: analysis } }))
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  reset: () => set({ bundle: null, requestAnalysis: {} }),
}))
