export type CaptureState = 'idle' | 'capturing' | 'paused' | 'stopped'
export type Importance = 'high' | 'medium' | 'low'
export type ScopeMode = 'all-traffic' | 'exact-host' | 'domain-and-subdomains'
export type BodyStorage = 'none' | 'inline' | 'file' | 'skipped'
export type BodyKind =
  | 'empty'
  | 'json'
  | 'form'
  | 'multipart'
  | 'text'
  | 'html'
  | 'xml'
  | 'binary'

export interface Header {
  name: string
  value: string
}

export interface QueryParam {
  name: string
  value: string
}

export interface CookiePair {
  name: string
  value: string
}

export interface BodyRef {
  storage: BodyStorage
  size: number
  is_text: boolean
  truncated: boolean
  path?: string
}

export interface BodyPayload {
  storage: BodyStorage
  size: number
  is_text: boolean
  truncated: boolean
  content: string | null
  kind: BodyKind
}

export interface CaptureConfig {
  mode: ScopeMode
  include_domains: string[]
  exclude_domains: string[]
  include_paths: string[]
  exclude_paths: string[]
  include_methods: string[]
  exclude_methods: string[]
  include_content_types: string[]
  exclude_content_types: string[]
  max_body_bytes: number
  capture_request_bodies: boolean
  capture_response_bodies: boolean
}

export interface Session {
  id: string
  name: string
  created_at: number
  updated_at: number
  status: CaptureState
  config: CaptureConfig
  request_count: number
  ignored_count: number
}

export interface SessionSummary {
  id: string
  name: string
  created_at: number
  updated_at: number
  status: CaptureState
  request_count: number
  ignored_count: number
  domains: string[]
}

export interface CaptureStatus {
  state: CaptureState
  session_id: string | null
  session_name: string | null
  proxy_addr: string | null
  captured: number
  ignored: number
}

export interface TimelineRow {
  id: string
  sequence_id: number
  timestamp: number
  method: string
  scheme: string
  host: string
  path: string
  query: string | null
  status: number | null
  duration_ms: number | null
  response_size: number
  family: string | null
  importance: Importance
  has_error: boolean
}

export interface TimelinePage {
  rows: TimelineRow[]
  total: number
  offset: number
}

export interface TimelineQuery {
  session_id: string
  search: string | null
  methods: string[]
  status_classes: number[]
  families: string[]
  hosts: string[]
  importance: string[]
  only_api: boolean
  only_errors: boolean
  only_json: boolean
  only_with_cookies: boolean
  only_with_body: boolean
  only_authenticated: boolean
  search_bodies: boolean
  limit: number
  offset: number
}

export interface CapturedResponse {
  status: number
  status_text: string
  protocol: string
  headers: Header[]
  body: BodyRef
  content_type: string | null
  family: string
  timestamp: number
  duration_ms: number
}

export interface CapturedRequest {
  id: string
  session_id: string
  sequence_id: number
  timestamp: number
  method: string
  url: string
  scheme: string
  host: string
  port: number
  path: string
  query: string | null
  normalized_path: string
  protocol: string
  client_addr: string | null
  remote_ip: string | null
  request_headers: Header[]
  request_body: BodyRef
  request_size: number
  request_content_type: string | null
  response: CapturedResponse | null
  error: string | null
  importance: Importance
  importance_reasons: string[]
}

export interface ResponseCookie {
  name: string
  value: string
  domain: string | null
  path: string
  secure: boolean
  http_only: boolean
  same_site: string | null
}

export interface CookieOrigin {
  name: string
  direction: string
  sequence_id: number
  value_preview: string
}

export interface RequestDetail {
  request: CapturedRequest
  query: QueryParam[]
  request_cookies: CookiePair[]
  response_cookies: ResponseCookie[]
  cookie_origins: CookieOrigin[]
}

export interface HostCount {
  host: string
  count: number
}
