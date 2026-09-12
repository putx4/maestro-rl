import { useEffect, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import {
  AlertTriangle,
  FileUp,
  FolderOpen,
  Loader2,
  Play,
  RotateCcw,
  Wand2,
} from 'lucide-react'
import { api, pickReplay } from '../lib/api'
import { useAppStore } from '../store'
import { durLabel } from '../lib/format'
import type { Analysis, CoachStage, ParseResult, ReplayFileInfo } from '../types'
import { AnalysisResult } from '../components/AnalysisResult'
import { PlayerTable } from '../components/PlayerTable'
import { ErrorBanner } from '../components/ui'

const STAGE_LABEL: Record<CoachStage, string> = {
  parsing: 'Leyendo el .replay…',
  connecting: 'Conectando con opencode (127.0.0.1:4096)…',
  analyzing: 'El Maestro está viendo tu replay…',
  saving: 'Guardando el análisis en tu historial…',
  done: '¡Listo!',
}

const whenLabel = (iso: string | null): string => {
  if (!iso) return '—'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  const now = new Date()
  const sameDay = d.toDateString() === now.toDateString()
  return sameDay
    ? `hoy ${d.toLocaleTimeString('es', { hour: '2-digit', minute: '2-digit' })}`
    : d.toLocaleDateString('es', { day: '2-digit', month: '2-digit' })
}

export function Analyze() {
  const { settings, refresh, setPage } = useAppStore()
  const [file, setFile] = useState<string | null>(null)
  const [parse, setParse] = useState<ParseResult | null>(null)
  const [analysis, setAnalysis] = useState<Analysis | null>(null)
  const [coachStage, setCoachStage] = useState<CoachStage | 'idle'>('idle')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [recent, setRecent] = useState<ReplayFileInfo[]>([])
  const [recentFolder, setRecentFolder] = useState<string | null>(null)
  const nickRef = useRef(settings.nick)

  useEffect(() => {
    api
      .recentReplays(20)
      .then((r) => {
        setRecent(r.replays)
        setRecentFolder(r.folder)
      })
      .catch(() => setRecent([]))
  }, [])

  useEffect(() => {
    if (nickRef.current !== settings.nick) {
      nickRef.current = settings.nick
      if (parse) {
        setFile(null)
        setParse(null)
        setAnalysis(null)
        setError(null)
      }
    }
  }, [settings.nick, parse])

  useEffect(() => {
    let unsub: (() => void) | undefined
    listen<CoachStage>('coach-stage', (e) => setCoachStage(e.payload)).then((u) => {
      unsub = u
    })
    return () => unsub?.()
  }, [])

  const me = parse?.summary.players.find(
    (p) => p.name.trim().toLowerCase() === settings.nick.trim().toLowerCase(),
  )

  const won = (() => {
    if (!parse || !me || parse.summary.teamScores.length !== 2) return null
    const [t0, t1] = parse.summary.teamScores
    const mine = me.team === 0 ? t0 : t1
    const theirs = me.team === 0 ? t1 : t0
    if (mine === theirs) return null
    return mine > theirs
  })()

  const loadFromPath = async (p: string) => {
    if (busy) return
    setError(null)
    setFile(p)
    setAnalysis(null)
    setBusy(true)
    setCoachStage('parsing')
    try {
      const r = await api.parseReplay(p)
      setParse(r)
    } catch (err) {
      setParse(null)
      setError(String(err))
    } finally {
      setBusy(false)
      setCoachStage('idle')
    }
  }

  const pick = async () => {
    const p = await pickReplay()
    if (p) await loadFromPath(p)
  }

  const analyze = async () => {
    if (!file) return
    setBusy(true)
    setCoachStage('connecting')
    setError(null)
    try {
      const a = await api.analyzeReplay(file)
      setAnalysis(a)
      await refresh()
    } catch (err) {
      setError(String(err))
    } finally {
      setBusy(false)
      setCoachStage('idle')
    }
  }

  const reset = () => {
    setFile(null)
    setParse(null)
    setAnalysis(null)
    setError(null)
  }

  return (
    <div className="grid" style={{ gap: 18 }}>
      {error && <ErrorBanner message={error} />}

      <div className="card">
        <div className="card-title">
          <h3>
            <FileUp size={18} /> Paso 1 — Elegí tu replay
          </h3>
          {file && (
            <button className="btn btn-ghost" onClick={reset} disabled={busy}>
              <RotateCcw size={15} /> Cambiar
            </button>
          )}
        </div>
        <div style={{ display: 'flex', gap: 12, alignItems: 'center', flexWrap: 'wrap' }}>
          <button className="btn btn-primary" onClick={pick} disabled={busy}>
            {busy ? <Loader2 className="spinner" size={16} /> : <FileUp size={17} />}
            {file ? 'Elegir otro…' : 'Elegir .replay'}
          </button>
          {file && (
            <span className="mono" style={{ color: 'var(--text)', wordBreak: 'break-all' }}>
              {file}
            </span>
          )}
        </div>
        <p className="progress-note" style={{ marginTop: 10 }}>
          Las replays están en{' '}
          <span className="kbd">…\\My Games\\Rocket League\\TAGame\\Demos</span>. El análisis corre
          100% local con opencode. Si el servidor no está activo, abrí una terminal y ejecutá{' '}
          <span className="kbd">opencode serve</span> (o ajustá la URL en Ajustes).
        </p>

        {recent.length > 0 && (
          <div style={{ marginTop: 14 }}>
            <div className="card-title" style={{ marginBottom: 8 }}>
              <h4 style={{ fontSize: 13, color: 'var(--text-dim)' }}>
                <FolderOpen size={14} /> Tus últimas replays
                {recentFolder && (
                  <span style={{ fontWeight: 400, marginLeft: 6 }}>({recentFolder})</span>
                )}
              </h4>
            </div>
            <ul className="recent-list">
              {recent.map((r) => (
                <li key={r.path}>
                  <button
                    className="btn btn-ghost"
                    style={{ width: '100%', justifyContent: 'space-between' }}
                    onClick={() => loadFromPath(r.path)}
                    disabled={busy}
                    title={r.path}
                  >
                    <span style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {r.name}
                    </span>
                    <span className="mono" style={{ color: 'var(--text-dim)', fontSize: 11 }}>
                      {whenLabel(r.modifiedAt)}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {file && parse && !analysis && (
        <div className="grid" style={{ gap: 18 }}>
          <div className="card">
            <div className="card-title">
              <h3>
                <Wand2 size={18} /> Paso 2 — Revisá lo que leímos
              </h3>
            </div>
            <div className="chip-line">
              <span className="pill pill-blue">{parse.summary.mapName ?? '?'}</span>
              <span className="pill pill-dim">{parse.summary.gameMode ?? '?'}</span>
              <span className="pill pill-dim">Duración {durLabel(parse.summary.numFrames)}</span>
              {parse.summary.date && (
                <span className="pill pill-dim">{parse.summary.date}</span>
              )}
              {parse.summary.teamScores.length === 2 && (
                <span className="pill pill-orange">
                  {parse.summary.teamScores[0]} — {parse.summary.teamScores[1]}
                </span>
              )}
              {won === true && <span className="pill pill-good">Victoria</span>}
              {won === false && <span className="pill pill-bad">Derrota</span>}
              {parse.summary.goals.length > 0 && (
                <span className="pill pill-dim">{parse.summary.goals.length} goles</span>
              )}
            </div>

            {!me && (
              <div className="banner" style={{ marginTop: 14 }}>
                <AlertTriangle size={16} />
                <span>
                  Tu nick <b>{settings.nick}</b> no aparece en esta replay. Revisá tu nick en
                  Ajustes para que el Maestro te identifique.
                </span>
              </div>
            )}

            <div className="divider" />
            <PlayerTable players={parse.summary.players} nick={settings.nick} />
          </div>

          <div className="card">
            <div className="card-title">
              <h3>
                <Play size={18} /> Paso 3 — Que el Maestro analice
              </h3>
            </div>
            <button className="btn btn-orange btn-block" onClick={analyze} disabled={busy}>
              {busy ? <Loader2 className="spinner" size={17} /> : <Play size={17} />}
              Analizar replay con el Maestro
            </button>
            <p className="progress-note" style={{ marginTop: 10 }}>
              Puede tardar uno o dos minutos. Con cada replay que le pases, el Maestro recuerda quién
              sos y cómo vas progresando.
            </p>
          </div>
        </div>
      )}

      {busy && coachStage !== 'idle' && (
        <div className="card">
          <div className="stage-line">
            <Loader2 className="spinner" size={22} />
            <div>
              <b>{STAGE_LABEL[coachStage as CoachStage]}</b>
              <div style={{ fontSize: 12.5, color: 'var(--text-dim)' }}>
                El modelo procesa tu partido… no cierres la app.
              </div>
            </div>
          </div>
        </div>
      )}

      {parse && analysis && !busy && (
        <div className="grid" style={{ gap: 14 }}>
          <div className="card">
            <div className="card-title">
              <h3 style={{ color: 'var(--teal)' }}>
                <Wand2 size={18} /> Veredicto del Maestro
              </h3>
              <div style={{ display: 'flex', gap: 8 }}>
                <button className="btn btn-orange" onClick={analyze} disabled={busy}>
                  <RotateCcw size={15} /> Re-analizar
                </button>
                <button className="btn btn-ghost" onClick={() => setPage('history')}>
                  Ver historial →
                </button>
              </div>
            </div>
          </div>
          <AnalysisResult analysis={analysis} />
        </div>
      )}
    </div>
  )
}