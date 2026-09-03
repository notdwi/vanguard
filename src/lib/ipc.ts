import { invoke } from '@tauri-apps/api/core'

import type {
  AnalysisBundle,
  AppSettings,
  BrowserOption,
  CaInfo,
  Comparison,
  ImportReport,
  RequestAnalysis,
  StorageInfo,
  TrustStorePlan,
} from '@/types/analysis'
import type {
  BodyPayload,
  CaptureConfig,
  CaptureStatus,
  HostCount,
  RequestDetail,
  Session,
  SessionSummary,
  TimelinePage,
  TimelineQuery,
} from '@/types/core'
import type { RepeaterDraft, ReplayOptions, ReplayResult } from '@/types/repeater'

export const api = {
  captureStatus: () => invoke<CaptureStatus>('capture_status'),
  startCapture: (sessionId: string, port?: number) =>
    invoke<CaptureStatus>('start_capture', { sessionId, port }),
  stopCapture: () => invoke<CaptureStatus>('stop_capture'),
  pauseCapture: (paused: boolean) => invoke<CaptureStatus>('pause_capture', { paused }),
  updateCaptureConfig: (sessionId: string, config: CaptureConfig) =>
    invoke<void>('update_capture_config', { sessionId, config }),
  flushCounters: () => invoke<CaptureStatus>('flush_counters'),
  listBrowsers: () => invoke<BrowserOption[]>('list_browsers'),
  launchBrowser: (browserId: string, url?: string) =>
    invoke<void>('launch_browser', { browserId, url }),
  clearBrowserProfile: (browserId: string) =>
    invoke<void>('clear_browser_profile', { browserId }),

  listSessions: () => invoke<SessionSummary[]>('list_sessions'),
  getSession: (sessionId: string) => invoke<Session>('get_session', { sessionId }),
  createSession: (name: string, config?: CaptureConfig) =>
    invoke<Session>('create_session', { name, config }),
  renameSession: (sessionId: string, name: string) =>
    invoke<void>('rename_session', { sessionId, name }),
  deleteSession: (sessionId: string) => invoke<void>('delete_session', { sessionId }),
  clearSession: (sessionId: string) => invoke<void>('clear_session', { sessionId }),
  sessionHosts: (sessionId: string) => invoke<HostCount[]>('session_hosts', { sessionId }),

  timeline: (query: TimelineQuery) => invoke<TimelinePage>('timeline', { query }),
  requestDetail: (requestId: string) => invoke<RequestDetail>('request_detail', { requestId }),
  loadBody: (requestId: string, side: 'request' | 'response', full = false) =>
    invoke<BodyPayload>('load_body', { requestId, side, full }),
  copyAsCurl: (requestId: string, maskSecrets = false) =>
    invoke<string>('copy_as_curl', { requestId, maskSecrets }),

  listDrafts: (sessionId: string) => invoke<RepeaterDraft[]>('list_drafts', { sessionId }),
  getDraft: (draftId: string) => invoke<RepeaterDraft>('get_draft', { draftId }),
  sendToRepeater: (requestId: string) =>
    invoke<RepeaterDraft>('send_to_repeater', { requestId }),
  newDraft: (sessionId: string) => invoke<RepeaterDraft>('new_draft', { sessionId }),
  saveDraft: (draft: RepeaterDraft) => invoke<RepeaterDraft>('save_draft', { draft }),
  deleteDraft: (draftId: string) => invoke<void>('delete_draft', { draftId }),
  runReplay: (draft: RepeaterDraft, options: ReplayOptions) =>
    invoke<ReplayResult[]>('run_replay', { draft, options }),
  listReplays: (draftId: string, limit?: number) =>
    invoke<ReplayResult[]>('list_replays', { draftId, limit }),
  clearReplays: (draftId: string) => invoke<void>('clear_replays', { draftId }),
  draftAsCurl: (draft: RepeaterDraft, maskSecrets = false) =>
    invoke<string>('draft_as_curl', { draft, maskSecrets }),
  compareResponses: (left: string, right: string) =>
    invoke<Comparison>('compare_responses', { left, right }),

  analyseSession: (sessionId: string, refresh = false) =>
    invoke<AnalysisBundle>('analyse_session', { sessionId, refresh }),
  analyseRequest: (sessionId: string, requestId: string) =>
    invoke<RequestAnalysis>('analyse_request', { sessionId, requestId }),
  endpointRequests: (sessionId: string, sequenceIds: number[]) =>
    invoke<{ request_ids: string[] }>('endpoint_requests', { sessionId, sequenceIds }),

  caInfo: () => invoke<CaInfo>('ca_info'),
  caPlan: () => invoke<TrustStorePlan>('ca_plan'),
  generateCa: () => invoke<CaInfo>('generate_ca'),
  installCa: () => invoke<CaInfo>('install_ca'),
  uninstallCa: () => invoke<CaInfo>('uninstall_ca'),
  deleteCa: () => invoke<CaInfo>('delete_ca'),
  exportCa: (destination: string) => invoke<string>('export_ca', { destination }),

  importHar: (path: string, name?: string) =>
    invoke<ImportReport>('import_har', { path, name }),
  exportHar: (sessionId: string, path: string) =>
    invoke<string>('export_har', { sessionId, path }),

  getSettings: () => invoke<AppSettings>('get_settings'),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>('save_settings', { settings }),
  storageInfo: () => invoke<StorageInfo>('storage_info'),
}

export function errorMessage(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}
