import { create } from 'zustand'

import { api, errorMessage } from '@/lib/ipc'
import type {
  CaptureConfig,
  CaptureStatus,
  HostCount,
  Session,
  SessionSummary,
} from '@/types/core'

import { useApp } from './app'

interface CaptureStore {
  status: CaptureStatus
  sessions: SessionSummary[]
  session: Session | null
  hosts: HostCount[]
  busy: boolean

  refreshStatus: () => Promise<void>
  refreshSessions: () => Promise<void>
  openSession: (sessionId: string) => Promise<void>
  createSession: (name: string) => Promise<Session | null>
  renameSession: (sessionId: string, name: string) => Promise<void>
  deleteSession: (sessionId: string) => Promise<void>
  clearSession: (sessionId: string) => Promise<void>
  updateConfig: (config: CaptureConfig) => Promise<void>
  start: () => Promise<void>
  stop: () => Promise<void>
  togglePause: () => Promise<void>
  refreshHosts: () => Promise<void>
  setStatus: (status: CaptureStatus) => void
}

const idle: CaptureStatus = {
  state: 'idle',
  session_id: null,
  session_name: null,
  proxy_addr: null,
  captured: 0,
  ignored: 0,
}

export const useCapture = create<CaptureStore>((set, get) => ({
  status: idle,
  sessions: [],
  session: null,
  hosts: [],
  busy: false,

  setStatus: (status) => set({ status }),

  refreshStatus: async () => {
    try {
      set({ status: await api.captureStatus() })
    } catch {
      /* backend not ready yet */
    }
  },

  refreshSessions: async () => {
    try {
      set({ sessions: await api.listSessions() })
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  openSession: async (sessionId) => {
    try {
      const session = await api.getSession(sessionId)
      set({ session })
      await get().refreshHosts()
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  createSession: async (name) => {
    try {
      const session = await api.createSession(name)
      await get().refreshSessions()
      set({ session })
      return session
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
      return null
    }
  },

  renameSession: async (sessionId, name) => {
    await api.renameSession(sessionId, name)
    await get().refreshSessions()
    if (get().session?.id === sessionId) await get().openSession(sessionId)
  },

  deleteSession: async (sessionId) => {
    await api.deleteSession(sessionId)
    if (get().session?.id === sessionId) set({ session: null, hosts: [] })
    await get().refreshSessions()
    await get().refreshStatus()
  },

  clearSession: async (sessionId) => {
    await api.clearSession(sessionId)
    await get().refreshSessions()
    await get().refreshHosts()
  },

  updateConfig: async (config) => {
    const session = get().session
    if (!session) return
    await api.updateCaptureConfig(session.id, config)
    set({ session: { ...session, config } })
  },

  start: async () => {
    const session = get().session
    const app = useApp.getState()
    if (!session) {
      app.notify(app.t('capture.needSession'), 'error')
      return
    }
    set({ busy: true })
    try {
      const status = await api.startCapture(session.id, app.settings.proxy_port)
      set({ status })
    } catch (error) {
      app.notify(errorMessage(error), 'error')
    } finally {
      set({ busy: false })
    }
  },

  stop: async () => {
    set({ busy: true })
    try {
      set({ status: await api.stopCapture() })
      await get().refreshSessions()
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    } finally {
      set({ busy: false })
    }
  },

  togglePause: async () => {
    const paused = get().status.state !== 'paused'
    try {
      set({ status: await api.pauseCapture(paused) })
    } catch (error) {
      useApp.getState().notify(errorMessage(error), 'error')
    }
  },

  refreshHosts: async () => {
    const id = get().session?.id
    if (!id) {
      set({ hosts: [] })
      return
    }
    try {
      set({ hosts: await api.sessionHosts(id) })
    } catch {
      set({ hosts: [] })
    }
  },
}))
