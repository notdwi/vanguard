import type { CookiePair, Header, QueryParam } from './core'

export type ReplayMode = 'sequential' | 'concurrent'

export interface RepeaterDraft {
  id: string
  session_id: string
  source_request_id: string | null
  source_sequence_id: number | null
  label: string
  method: string
  url: string
  query: QueryParam[]
  headers: Header[]
  cookies: CookiePair[]
  body: string
  created_at: number
  updated_at: number
}

export interface ReplayOptions {
  iterations: number
  mode: ReplayMode
  delay_ms: number
  follow_redirects: boolean
  timeout_ms: number
}

export interface RepeaterSnapshot {
  method: string
  url: string
  headers: Header[]
  body: string
}

export interface ReplayResult {
  id: string
  draft_id: string
  session_id: string
  index: number
  started_at: number
  duration_ms: number
  status: number | null
  status_text: string
  protocol: string
  headers: Header[]
  body: string | null
  body_size: number
  body_is_text: boolean
  content_type: string | null
  error: string | null
  sent: RepeaterSnapshot
}

export interface ReplayProgress {
  draft_id: string
  completed: number
  total: number
}
