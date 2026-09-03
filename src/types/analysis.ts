import type { Header } from './core'

export type TokenKind =
  | 'bearer'
  | 'jwt'
  | 'api-key'
  | 'csrf'
  | 'session-id'
  | 'request-id'
  | 'basic'
  | 'unknown'

export type TokenSource = 'header' | 'cookie' | 'query' | 'body'

export type LinkKind =
  | 'cookie'
  | 'header-value'
  | 'query-value'
  | 'body-value'
  | 'json-value'
  | 'path-value'

export interface SessionAnalysis {
  requests: number
  domains: number
  api_endpoints: number
  unique_endpoints: number
  json_responses: number
  post_requests: number
  with_cookies: number
  possible_tokens: number
  high_importance: number
  errors: number
  total_bytes: number
}

export interface EndpointGroup {
  normalized: string
  host: string
  methods: string[]
  count: number
  is_api: boolean
  sample_request_id: string
  sequence_ids: number[]
  status_codes: number[]
  avg_duration_ms: number
}

export interface CookieEvent {
  request_id: string
  sequence_id: number
  method: string
  path: string
  value_preview: string
}

export interface CookieUsage {
  name: string
  domain: string
  value_preview: string
  distinct_values: number
  created_by: CookieEvent[]
  used_by: CookieEvent[]
}

export interface DetectedToken {
  kind: TokenKind
  source: TokenSource
  name: string
  value_preview: string
  value_hash: string
  used_by: number[]
  first_seen_request_id: string
}

export interface Relationship {
  from_request_id: string
  from_sequence_id: number
  from_path: string
  to_request_id: string
  to_sequence_id: number
  to_path: string
  kind: LinkKind
  value_preview: string
  source_json_path: string | null
  target_location: string
  confidence: number
}

export interface FlowNode {
  id: string
  label: string
  host: string
  method: string
  count: number
  importance: string
  depth: number
  sample_request_id: string
}

export interface FlowEdge {
  from: string
  to: string
  kind: LinkKind
  label: string
  weight: number
}

export interface FlowGraph {
  nodes: FlowNode[]
  edges: FlowEdge[]
}

export interface AnalysisBundle {
  session_id: string
  generated_at: number
  overview: SessionAnalysis
  endpoints: EndpointGroup[]
  tokens: DetectedToken[]
  relationships: Relationship[]
  graph: FlowGraph
  cookies: CookieUsage[]
  truncated: boolean
}

export interface DetectedId {
  value: string
  location: string
  kind: string
}

export interface RequestAnalysis {
  importance: string
  reasons: string[]
  normalized_endpoint: string
  is_api: boolean
  detected_ids: DetectedId[]
  tokens: DetectedToken[]
  inbound: Relationship[]
  outbound: Relationship[]
  repeat_count: number
}

export interface CaInfo {
  exists: boolean
  common_name: string
  cert_path: string
  key_path: string
  fingerprint: string | null
  not_after: string | null
  installed: boolean
}

export interface TrustStorePlan {
  platform: string
  steps: string[]
  requires_elevation: boolean
  manual_instructions: string[]
}

export type BrowserKind = 'chromium' | 'firefox'

export interface BrowserOption {
  id: string
  name: string
  path: string
  kind: BrowserKind
  uses_system_trust: boolean
}

export interface AppSettings {
  language: string
  proxy_port: number
  mask_secrets: boolean
  timeline_page_size: number
  auto_analyse: boolean
  default_replay_delay_ms: number
  sensitive_headers: string[]
}

export interface StorageInfo {
  data_dir: string
  database_bytes: number
  blob_bytes: number
}

export interface ImportReport {
  session_id: string
  imported: number
  skipped: number
}

export interface ComparisonSide {
  label: string
  status: number | null
  duration_ms: number
  size: number
  content_type: string | null
  headers: Header[]
  body: string | null
}

export interface DiffEntry {
  kind: 'added' | 'removed' | 'changed'
  path: string
  left: string | null
  right: string | null
  volatile: boolean
}

export interface Comparison {
  left: ComparisonSide
  right: ComparisonSide
  header_diff: DiffEntry[]
  body_diff: DiffEntry[]
  body_comparable: boolean
}
