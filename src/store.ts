import { create } from 'zustand'
import { api } from './lib/api'
import { defaultSettings } from './types'
import type {
  CoachSettings,
  EntryMeta,
  PageId,
  ProgressReport,
} from './types'

interface AppStore {
  page: PageId
  setPage: (page: PageId) => void

  settings: CoachSettings
  settingsLoaded: boolean
  entries: EntryMeta[]
  progress: ProgressReport | null
  loaded: boolean
  error: string | null

  init: () => Promise<void>
  refresh: () => Promise<void>
  setError: (err: string | null) => void
  saveSettings: (settings: CoachSettings) => Promise<void>
  deleteEntry: (id: string) => Promise<void>
  clearHistory: () => Promise<void>
}

export const useAppStore = create<AppStore>((set, get) => ({
  page: 'dashboard',
  setPage: (page) => set({ page }),

  settings: defaultSettings(),
  settingsLoaded: false,
  entries: [],
  progress: null,
  loaded: false,
  error: null,

  init: async () => {
    try {
      const settings = await api.getSettings()
      const [entries, progress] = await Promise.all([
        api.listEntries(),
        api.getProgress(),
      ])
      set({
        settings,
        settingsLoaded: true,
        entries,
        progress,
        loaded: true,
        error: null,
      })
    } catch (err) {
      set({ error: stringifyErr(err) })
    }
  },

  refresh: async () => {
    try {
      const [entries, progress] = await Promise.all([
        api.listEntries(),
        api.getProgress(),
      ])
      set({ entries, progress, loaded: true, error: null })
    } catch (err) {
      set({ error: stringifyErr(err) })
    }
  },

  setError: (err) => set({ error: err }),

  saveSettings: async (settings) => {
    await api.saveSettings(settings)
    set({ settings, settingsLoaded: true })
  },

  deleteEntry: async (id) => {
    await api.deleteEntry(id)
    await get().refresh()
  },

  clearHistory: async () => {
    await api.clearHistory()
    await get().refresh()
  },
}))

export function stringifyErr(err: unknown): string {
  if (typeof err === 'string') return err
  if (err instanceof Error) return err.message
  return String(err)
}