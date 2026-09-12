import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import type {
  Analysis,
  AnalysisEntry,
  CoachSettings,
  EntryMeta,
  ParseResult,
  ProgressReport,
  RecentReplays,
  TestResult,
} from '../types'

export async function pickReplay(): Promise<string | null> {
  const file = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Replays de Rocket League', extensions: ['replay'] }],
  })
  if (!file || Array.isArray(file)) return null
  return file
}

export const api = {
  getSettings: () => invoke<CoachSettings>('get_settings'),
  saveSettings: (settings: CoachSettings) =>
    invoke<void>('save_settings', { settings }),
  parseReplay: (path: string) => invoke<ParseResult>('parse_replay', { path }),
  analyzeReplay: (path: string) =>
    invoke<Analysis>('analyze_replay', { path }),
  reanalyzeEntry: (id: string) =>
    invoke<Analysis>('reanalyze_entry', { id }),
  listEntries: () => invoke<EntryMeta[]>('list_entries'),
  getEntry: (id: string) => invoke<AnalysisEntry>('get_entry', { id }),
  deleteEntry: (id: string) => invoke<void>('delete_entry', { id }),
  clearHistory: () => invoke<void>('clear_history'),
  getProgress: () => invoke<ProgressReport>('get_progress'),
  testConnection: () => invoke<TestResult>('test_connection'),
  recentReplays: (limit: number) =>
    invoke<RecentReplays>('recent_replays', { limit }),
}