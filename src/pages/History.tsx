import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { ArrowLeft, FileUp, History as HistoryIcon, Loader2, RotateCcw, Trash2 } from 'lucide-react'
import { api } from '../lib/api'
import { useAppStore } from '../store'
import { fullDate } from '../lib/format'
import type { AnalysisEntry, CoachStage, EntryMeta } from '../types'
import { AnalysisResult } from '../components/AnalysisResult'
import { PlayerTable } from '../components/PlayerTable'
import { EmptyState, ErrorBanner, Loading } from '../components/ui'

export function History() {
  const { entries, settings, refresh, deleteEntry, setPage, loaded, error } = useAppStore()
  const [selected, setSelected] = useState<AnalysisEntry | null>(null)
  const [busyId, setBusyId] = useState<string | null>(null)
  const [coachStage, setCoachStage] = useState<CoachStage | 'idle'>('idle')
  const [actionError, setActionError] = useState<string | null>(null)

  useEffect(() => {
    let unsub: (() => void) | undefined
    listen<CoachStage>('coach-stage', (e) => setCoachStage(e.payload)).then((u) => {
      unsub = u
    })
    return () => unsub?.()
  }, [])

  const open = async (id: string) => {
    setActionError(null)
    try {
      const e = await api.getEntry(id)
      setSelected(e)
    } catch (err) {
      setActionError(String(err))
    }
  }

  const reanalyze = async (id: string) => {
    setActionError(null)
    setBusyId(id)
    try {
      await api.reanalyzeEntry(id)
      await refresh()
      const e = await api.getEntry(id)
      setSelected(e)
    } catch (err) {
      setActionError(String(err))
    } finally {
      setBusyId(null)
      setCoachStage('idle')
    }
  }

  const remove = async (id: string) => {
    if (!confirm('¿Borrar este análisis del historial?')) return
    setActionError(null)
    try {
      await deleteEntry(id)
      if (selected?.id === id) setSelected(null)
    } catch (err) {
      setActionError(String(err))
    }
  }

  if (!loaded) return <Loading />

  return (
    <div className="grid" style={{ gap: 18 }}>
      {error && <ErrorBanner message={error} />}
      {actionError && <ErrorBanner message={actionError} />}

      {selected && (
        <div className="grid" style={{ gap: 14 }}>
          <div className="card" style={{ padding: 16 }}>
            <div
              style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}
            >
              <button className="btn btn-ghost" onClick={() => setSelected(null)}>
                <ArrowLeft size={16} /> Volver
              </button>
              <span className="entry-name" style={{ flex: 1 }}>
                {selected.fileName}
              </span>
              <span className="pill pill-dim">{fullDate(selected.createdAt)}</span>
              {selected.summary.teamScores.length === 2 && (
                <span className="pill pill-orange">
                  {selected.summary.teamScores[0]} — {selected.summary.teamScores[1]}
                </span>
              )}
              <button
                className="btn btn-orange"
                onClick={() => reanalyze(selected.id)}
                disabled={busyId === selected.id}
              >
                {busyId === selected.id ? (
                  <Loader2 className="spinner" size={15} />
                ) : (
                  <RotateCcw size={15} />
                )}
                Re-analizar
              </button>
              <button className="btn btn-danger" onClick={() => remove(selected.id)}>
                <Trash2 size={15} />
              </button>
            </div>
            {busyId === selected.id && coachStage !== 'idle' && (
              <div style={{ marginTop: 12 }}>
                <div className="stage-line">
                  <Loader2 className="spinner" size={18} />
                  <span>{coachStage === 'connecting' ? 'Conectando con opencode…' : coachStage === 'analyzing' ? 'El Maestro re-analizando…' : 'Guardando…'}</span>
                </div>
              </div>
            )}
            <div className="divider" />
            <div className="chip-line" style={{ marginBottom: 14 }}>
              <span className="pill pill-blue">{selected.summary.mapName ?? '?'}</span>
              <span className="pill pill-dim">{selected.summary.gameMode ?? '?'}</span>
              {selected.summary.date && (
                <span className="pill pill-dim">{selected.summary.date}</span>
              )}
              {selected.summary.goals.length > 0 && (
                <span className="pill pill-dim">{selected.summary.goals.length} goles</span>
              )}
            </div>
            <PlayerTable players={selected.summary.players} nick={settings.nick} />
          </div>

          {selected.analysis && <AnalysisResult analysis={selected.analysis} />}
        </div>
      )}

      {!selected && (
        <div className="card" style={{ padding: 10 }}>
          {entries.length === 0 ? (
            <EmptyState
              icon={<HistoryIcon size={38} />}
              title="Sin replays todavía"
              subtitle="Cada análisis se guarda acá para que el Maestro aprenda de vos."
            >
              <button className="btn btn-primary" onClick={() => setPage('analyze')}>
                <FileUp size={16} /> Analizar la primera
              </button>
            </EmptyState>
          ) : (
            <div className="grid" style={{ gap: 10, padding: 8 }}>
              {entries.map((e) => (
                <EntryRow key={e.id} e={e} onOpen={() => open(e.id)} onReanalyze={() => reanalyze(e.id)} onDelete={() => remove(e.id)} busy={busyId === e.id} />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function EntryRow({
  e,
  onOpen,
  onReanalyze,
  onDelete,
  busy,
}: {
  e: EntryMeta
  onOpen: () => void
  onReanalyze: () => void
  onDelete: () => void
  busy: boolean
}) {
  return (
    <div className="entry-row" onClick={onOpen}>
      <div style={{ minWidth: 0 }}>
        <div className="entry-name">{e.fileName}</div>
        <div className="entry-meta">{fullDate(e.createdAt)}</div>
      </div>
      <div className="entry-meta">
        <span>{e.mapName ?? '?'}</span>
        <span>{e.gameMode ?? '?'}</span>
      </div>
      <div>
        {(e.teamScores ?? []).length === 2 && (
          <span className="pill pill-orange">
            {e.teamScores[0]} — {e.teamScores[1]}
          </span>
        )}
        {e.won === true && <span className="pill pill-good" style={{ marginLeft: 6 }}>V</span>}
        {e.won === false && <span className="pill pill-bad" style={{ marginLeft: 6 }}>D</span>}
      </div>
      <div>
        {e.analysis ? (
          <div className="score-big" style={{ color: e.analysis.score >= 70 ? 'var(--good)' : e.analysis.score >= 50 ? 'var(--warn)' : 'var(--bad)' }}>
            {Math.round(e.analysis.score)}
          </div>
        ) : (
          <span className="pill pill-dim">pendiente</span>
        )}
      </div>
      <div style={{ display: 'flex', gap: 6 }} onClick={(ev) => ev.stopPropagation()}>
        <button
          className="btn btn-ghost"
          style={{ padding: '7px 10px' }}
          onClick={onReanalyze}
          disabled={busy}
          title="Re-analizar con el Maestro"
        >
          {busy ? <Loader2 className="spinner" size={15} /> : <RotateCcw size={15} />}
        </button>
        <button
          className="btn btn-ghost"
          style={{ padding: '7px 10px', color: 'var(--bad)' }}
          onClick={onDelete}
          title="Borrar"
        >
          <Trash2 size={15} />
        </button>
      </div>
    </div>
  )
}