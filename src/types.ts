export interface CoachSettings {
  serverUrl: string
  nick: string
  rank: string
  modes: string
  language: string
  memoryDepth: number
  timeoutSecs: number
}

export const defaultSettings = (): CoachSettings => ({
  serverUrl: 'http://127.0.0.1:4096',
  nick: 'lacuca3000',
  rank: 'Campeón 1',
  modes: '2v2 y 3v3',
  language: 'es',
  memoryDepth: 10,
  timeoutSecs: 180,
})

export interface PlayerStat {
  name: string
  team: number
  goals: number
  assists: number
  saves: number
  shots: number
  demolishes: number
  boostPickups: number
  boostBigPads: number
  boostSmallPads: number
  avgBoost: number
  lowBoostPct: number
  clears: number
  denials: number
  steals: number
  breakoutDamage: number
  matchScore: number
  ownGoals: number
}

export interface ReplayFileInfo {
  path: string
  name: string
  modifiedAt: string | null
  sizeBytes: number
}

export interface RecentReplays {
  folder: string | null
  replays: ReplayFileInfo[]
}

export interface GoalEvent {
  frame: number
  playerName: string
  team: number
  ownGoal: boolean
}

export interface ReplaySummary {
  majorVersion: number
  gameType: string
  mapName: string | null
  gameMode: string | null
  matchType: string | null
  teamSize: number | null
  teamScores: number[]
  date: string | null
  numFrames: number | null
  goals: GoalEvent[]
  players: PlayerStat[]
}

export interface CategoryScore {
  key: string
  name: string
  nota: number
  comentario: string
}

export interface Consejo {
  titulo: string
  explicacion: string
  drill: string
}

export interface ProgresoRelativo {
  mejoras: string[]
  regresiones: string[]
}

export interface Analysis {
  resumen: string
  puntuacion: number
  categorias: CategoryScore[]
  aciertos: string[]
  errores: string[]
  consejos: Consejo[]
  progreso: ProgresoRelativo
}

export interface AnalysisSummary {
  score: number
  categoryCount: number
}

export interface EntryMeta {
  id: string
  createdAt: string
  fileName: string
  mapName: string | null
  gameMode: string | null
  teamScores: number[]
  myStats: PlayerStat | null
  won: boolean | null
  analysis: AnalysisSummary | null
}

export interface AnalysisEntry {
  id: string
  createdAt: string
  filePath: string
  fileName: string
  summary: ReplaySummary
  rawProperties: unknown
  analysis: Analysis | null
  rawResponse: string | null
}

export interface ParseResult {
  summary: ReplaySummary
  rawProperties: unknown
}

export interface TestResult {
  ok: boolean
  message: string
}

export interface CategoryTrend {
  key: string
  name: string
  appearances: number
  last: number | null
  prevAvg: number | null
  delta: number | null
}

export interface ProgressPoint {
  id: string
  createdAt: string
  fileName: string
  score: number
}

export interface ProgressReport {
  total: number
  analyzed: number
  wins: number
  losses: number
  categories: CategoryTrend[]
  history: ProgressPoint[]
}

export type PageId = 'dashboard' | 'analyze' | 'history' | 'progress' | 'settings'

export type CoachStage = 'parsing' | 'connecting' | 'analyzing' | 'saving' | 'done'