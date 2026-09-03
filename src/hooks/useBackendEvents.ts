import { useEffect } from 'react'

import { listen, type UnlistenFn } from '@tauri-apps/api/event'

import { useCapture } from '@/stores/capture'
import { useRepeater } from '@/stores/repeater'
import { useTimeline } from '@/stores/timeline'
import type { CaptureStatus, TimelineRow } from '@/types/core'
import type { ReplayProgress, ReplayResult } from '@/types/repeater'

interface RequestStarted {
  session_id: string
  request_id: string
  sequence_id: number
  timestamp: number
  method: string
  scheme: string
  host: string
  path: string
  query: string | null
  protocol: string
  importance: TimelineRow['importance']
}

interface ResponseReceived {
  session_id: string
  request_id: string
  sequence_id: number
  status: number
  content_type: string | null
  family: string
  body_size: number
  duration_ms: number
  importance: TimelineRow['importance']
}

interface RequestFailed {
  session_id: string
  request_id: string
  message: string
}

/// Keeps the timeline and repeater in sync with the backend event bus.
export function useBackendEvents() {
  useEffect(() => {
    const unlisteners: Promise<UnlistenFn>[] = []

    unlisteners.push(
      listen<RequestStarted>('request:started', (event) => {
        const timeline = useTimeline.getState()
        const sessionId = useCapture.getState().session?.id
        if (sessionId && event.payload.session_id !== sessionId) return
        timeline.applyLiveRow({
          id: event.payload.request_id,
          sequence_id: event.payload.sequence_id,
          timestamp: event.payload.timestamp,
          method: event.payload.method,
          scheme: event.payload.scheme,
          host: event.payload.host,
          path: event.payload.path,
          query: event.payload.query,
          status: null,
          duration_ms: null,
          response_size: 0,
          family: null,
          importance: event.payload.importance,
          has_error: false,
        })
      }),
    )

    unlisteners.push(
      listen<ResponseReceived>('response:received', (event) => {
        useTimeline.getState().patchRow(event.payload.request_id, {
          status: event.payload.status,
          duration_ms: event.payload.duration_ms,
          response_size: event.payload.body_size,
          family: event.payload.family,
          importance: event.payload.importance,
        })
      }),
    )

    unlisteners.push(
      listen<RequestFailed>('request:failed', (event) => {
        useTimeline.getState().patchRow(event.payload.request_id, { has_error: true })
      }),
    )

    unlisteners.push(
      listen<CaptureStatus>('capture:status', (event) => {
        useCapture.getState().setStatus(event.payload)
      }),
    )

    unlisteners.push(
      listen<ReplayResult>('replay:result', (event) => {
        useRepeater.getState().pushResult(event.payload)
      }),
    )

    unlisteners.push(
      listen<ReplayProgress>('replay:progress', (event) => {
        useRepeater
          .getState()
          .setProgress({ completed: event.payload.completed, total: event.payload.total })
      }),
    )

    unlisteners.push(
      listen('sessions:changed', () => {
        void useCapture.getState().refreshSessions()
      }),
    )

    return () => {
      unlisteners.forEach((p) => void p.then((fn) => fn()))
    }
  }, [])
}

/// Periodically syncs the in-memory capture counters with the session row.
export function useCounterSync(active: boolean) {
  useEffect(() => {
    if (!active) return
    const id = window.setInterval(() => {
      void useCapture.getState().refreshStatus()
    }, 2000)
    return () => window.clearInterval(id)
  }, [active])
}
