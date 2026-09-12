import { ArrowDownRight, ArrowUpRight, FileUp, Radar, Share2, Trophy, Wand2 } from 'lucide-react'
import { useAppStore } from '../store'
import { shortDate } from '../lib/format'
import { EmptyState, ErrorBanner, Loading, ScoreRing, Stat } from '../components/ui'

export function Dashboard() {
  const { setPage: go, entries, progress, loaded, error, settings } = useAppStore()

  if (!loaded) return <Loading />

  const last = entries[0]
  const analyzed = progress?.analyzed ?? 0
  const wins = progress?.wins ?? 0
  const losses = progress?.losses ?? 0

  const withDelta = (progress?.categories ?? [])
    .filter((c) => c.delta !== null && c.last !== null)
    .sort((a, b) => (b.delta ?? 0) - (a.delta ?? 0))
  const bestDelta = withDelta[0]
  const weakest = (progress?.categories ?? [])
    .filter((c) => c.last !== null)
    .sort((a, b) => (a.last ?? 0) - (b.last ?? 0))[0]

  const hist = progress?.history ?? []
  const avg = hist.length > 0 ? hist.reduce((acc, p) => acc + p.score, 0) / hist.length : null

  return (
    <div className="grid" style={{ gap: 20 }}>
      {error && <ErrorBanner message={error} />}

      <div className="card" style={{ position: 'relative', overflow: 'hidden' }}>
        <div className="grid grid-2" style={{ alignItems: 'center', gap: 30 }}>
          <div>
            <h2 style={{ marginBottom: 10 }}>
              Hola {settings.nick.split(/[ _]/)[0] || 'jugador'}, ¿subimos una replay?
            </h2>
            <p style={{ color: 'var(--text-dim)', marginBottom: 20, maxWidth: 540 }}>
              Pasá una replay, el <b style={{ color: 'var(--orange)' }}>Maestro</b> la analiza con
              opencode en tu máquina (sin pagar nada) y te dice qué hiciste bien, qué fallaste y{' '}
              <b>qué entrenar</b> para romper el techo del {settings.rank}.
            </p>
            <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
              <button className="btn btn-primary" onClick={() => go('analyze')}>
                <FileUp size={17} /> Analizar replay
              </button>
              <button className="btn btn-ghost" onClick={() => go('settings')}>
                Configurar
              </button>
            </div>
          </div>
          <div
            style={{
              display: 'grid',
              placeItems: 'center',
              gap: 8,
              color: 'var(--blue)',
              opacity: 0.9,
            }}
          >
            <Radar size={92} strokeWidth={1.2} style={{ opacity: 0.5 }} />
            <span
              className="pill pill-blue"
              style={{ letterSpacing: 1.6, textTransform: 'uppercase', fontSize: 11 }}
            >
              Tu coach personal
            </span>
          </div>
        </div>
      </div>

      <div className="grid grid-4">
        <Stat
          label="Replays analizadas"
          value={analyzed}
          sub={`${progress?.total ?? 0} guardadas en total`}
          icon={<Wand2 size={14} />}
        />
        <Stat
          label="Victorias / Derrotas"
          value={`${wins} / ${losses}`}
          sub="según recreaciones de la replay"
          icon={<Trophy size={14} />}
        />
        <Stat
          label="Nota promedio"
          value={avg !== null ? Math.round(avg) : '—'}
          sub="puntuación del Maestro"
          icon={<Radar size={14} />}
        />
        {weakest ? (
          <Stat
            label="Categoría más débil"
            value={weakest.name}
            sub={`última nota: ${Math.round(weakest.last ?? 0)}`}
            icon={<ArrowDownRight size={14} />}
          />
        ) : (
          <Stat label="Categoría más débil" value="—" sub="analizá tu primera replay" />
        )}
      </div>

      {last ? (
        <div className="card">
          <div className="card-title">
            <h3>
              <Share2 size={18} /> Último análisis
            </h3>
            <span className="pill pill-dim">{shortDate(last.createdAt)}</span>
          </div>
          <div className="grid grid-2" style={{ alignItems: 'center', gap: 24 }}>
            <div className="grid" style={{ gap: 8 }}>
              <div className="entry-name">
                {last.fileName}
                {last.mapName && <span className="pill pill-dim">{last.mapName}</span>}
              </div>
              <div className="chip-line">
                {(last.teamScores ?? []).length === 2 && (
                  <span className="pill pill-orange">
                    {last.teamScores[0]} — {last.teamScores[1]}
                  </span>
                )}
                {last.won === true && <span className="pill pill-good">Victoria</span>}
                {last.won === false && <span className="pill pill-bad">Derrota</span>}
                {last.myStats && (
                  <span className="pill pill-dim">
                    Tuyas: {last.myStats.goals}G · {last.myStats.assists}A · {last.myStats.saves}S ·{' '}
                    {last.myStats.shots}T
                  </span>
                )}
              </div>
            </div>
            {last.analysis && (
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 16,
                  justifyContent: 'flex-end',
                }}
              >
                <ScoreRing score={last.analysis.score} size={84} />
                <button className="btn btn-ghost" onClick={() => go('history')}>
                  Ver historial →
                </button>
              </div>
            )}
          </div>

          {bestDelta && bestDelta.delta !== 0 && (
            <>
              <div className="divider" />
              <div className="result-line">
                <span className={`pill ${(bestDelta.delta ?? 0) > 0 ? 'pill-good' : 'pill-bad'}`}>
                  {(bestDelta.delta ?? 0) > 0 ? (
                    <ArrowUpRight size={14} />
                  ) : (
                    <ArrowDownRight size={14} />
                  )}
                  {bestDelta.name}: {(bestDelta.delta ?? 0) > 0 ? '+' : ''}
                  {(bestDelta.delta ?? 0).toFixed(1)} pts vs tu historial
                </span>
                <span className="progress-note">Es lo que más está cambiando entre tus replays.</span>
              </div>
            </>
          )}
        </div>
      ) : (
        <div className="card">
          <EmptyState
            icon={<Radar size={38} />}
            title="Todavía no hay análisis"
            subtitle="Subí tu primera replay para que el Maestro arranque a conocerte."
          >
            <button className="btn btn-primary" onClick={() => go('analyze')}>
              <FileUp size={16} /> Subir replay
            </button>
          </EmptyState>
        </div>
      )}
    </div>
  )
}