import { useEffect, useState } from 'react'
import {
  ArrowDownRight,
  ArrowUpRight,
  LineChart as LineIcon,
  Radar,
  Trophy,
  Wand2,
} from 'lucide-react'
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { api } from '../lib/api'
import { useAppStore } from '../store'
import { shortDate } from '../lib/format'
import type { AnalysisEntry } from '../types'
import { EmptyState, ErrorBanner, Loading, Stat } from '../components/ui'

const TOOLTIP_STYLE = {
  background: '#0d1120',
  border: '1px solid rgba(255,255,255,0.12)',
  borderRadius: 10,
  color: '#c9d1e2',
  fontSize: 12.5,
}

function DeltaPill({ delta }: { delta: number | null }) {
  if (delta === null) return <span className="pill pill-dim">—</span>
  if (delta > 0.1)
    return (
      <span className="pill pill-good">
        <ArrowUpRight size={13} /> +{delta.toFixed(1)}
      </span>
    )
  if (delta < -0.1)
    return (
      <span className="pill pill-bad">
        <ArrowDownRight size={13} /> {delta.toFixed(1)}
      </span>
    )
  return <span className="pill pill-dim">estable</span>
}

export function Progress() {
  const { entries, progress, loaded, error, setPage } = useAppStore()
  const [last, setLast] = useState<AnalysisEntry | null>(null)

  useEffect(() => {
    let alive = true
    if (entries[0]) {
      api
        .getEntry(entries[0].id)
        .then((e) => {
          if (alive) setLast(e)
        })
        .catch(() => {})
    } else {
      setLast(null)
    }
    return () => {
      alive = false
    }
  }, [entries])

  if (!loaded) return <Loading />

  const cats = [...(progress?.categories ?? [])]
  const withLast = cats.filter((c) => c.last !== null)
  const bestCat = [...withLast].sort((a, b) => (b.last ?? 0) - (a.last ?? 0))[0]
  const weakestCat = [...withLast].sort((a, b) => (a.last ?? 0) - (b.last ?? 0))[0]

  const hist = progress?.history ?? []
  const avg = hist.length > 0 ? hist.reduce((a, p) => a + p.score, 0) / hist.length : null

  const lineData = hist.map((p) => ({
    name: `${shortDate(p.createdAt)}\n${p.fileName}`,
    score: Math.round(p.score),
  }))

  const barData = [...cats]
    .sort((a, b) => (a.last ?? 0) - (b.last ?? 0))
    .map((c) => ({ name: c.name, nota: Math.round(c.last ?? 0) }))

  return (
    <div className="grid" style={{ gap: 18 }}>
      {error && <ErrorBanner message={error} />}

      {hist.length === 0 ? (
        <div className="card">
          <EmptyState
            icon={<LineIcon size={38} />}
            title="Todavía no hay progreso que graficar"
            subtitle="Analizá al menos dos replays y acá vas a ver tu evolución categoría por categoría."
          >
            <button className="btn btn-primary" onClick={() => setPage('analyze')}>
              <Wand2 size={16} /> Analizar replay
            </button>
          </EmptyState>
        </div>
      ) : (
        <>
          <div className="grid grid-4">
            <Stat label="Replays analizadas" value={hist.length} icon={<Radar size={14} />} />
            <Stat
              label="Promedio de nota"
              value={avg !== null ? Math.round(avg) : '—'}
              icon={<Wand2 size={14} />}
            />
            <Stat
              label="Victorias / Derrotas"
              value={`${progress?.wins ?? 0} / ${progress?.losses ?? 0}`}
              icon={<Trophy size={14} />}
            />
            {bestCat ? (
              <Stat
                label="Lo mejor tuyo"
                value={bestCat.name}
                sub={`última nota: ${Math.round(bestCat.last ?? 0)}`}
                icon={<ArrowUpRight size={14} />}
              />
            ) : (
              <Stat label="Lo mejor tuyo" value="—" />
            )}
          </div>

          <div className="card">
            <div className="card-title">
              <h3>
                <LineIcon size={18} /> Evolución de tu nota general
              </h3>
            </div>
            <ResponsiveContainer width="100%" height={250}>
              <LineChart data={lineData} margin={{ top: 8, right: 12, left: -18, bottom: 4 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.06)" />
                <XAxis
                  dataKey="name"
                  stroke="var(--text-dim)"
                  fontSize={11}
                  tickLine={false}
                  axisLine={false}
                  interval="preserveStartEnd"
                />
                <YAxis
                  domain={[0, 100]}
                  stroke="var(--text-dim)"
                  fontSize={11}
                  tickLine={false}
                  axisLine={false}
                />
                <Tooltip contentStyle={TOOLTIP_STYLE} />
                <Line
                  type="monotone"
                  dataKey="score"
                  stroke="var(--blue)"
                  strokeWidth={3}
                  dot={{ r: 4, fill: 'var(--blue)', strokeWidth: 0 }}
                  activeDot={{ r: 6, fill: 'var(--orange)' }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>

          <div className="grid grid-2">
            <div className="card">
              <div className="card-title">
                <h3>
                  <Wand2 size={18} /> Nota por categoría (última replay)
                </h3>
              </div>
              {barData.length === 0 ? (
                <p className="progress-note">Sin categorías todavía.</p>
              ) : (
                <ResponsiveContainer width="100%" height={220}>
                  <BarChart data={barData} margin={{ top: 8, right: 8, left: -18, bottom: 8 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.06)" vertical={false} />
                    <XAxis dataKey="name" stroke="var(--text-dim)" fontSize={11} tickLine={false} axisLine={false} />
                    <YAxis domain={[0, 100]} stroke="var(--text-dim)" fontSize={11} tickLine={false} axisLine={false} />
                    <Tooltip contentStyle={TOOLTIP_STYLE} cursor={{ fill: 'rgba(255,255,255,0.04)' }} />
                    <Bar dataKey="nota" radius={[6, 6, 0, 0]}>
                      {barData.map((d, i) => (
                        <Cell key={i} fill={d.nota >= 75 ? 'var(--good)' : d.nota >= 55 ? 'var(--warn)' : 'var(--bad)'} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
              )}
            </div>

            <div className="card">
              <div className="card-title">
                <h3>
                  <Radar size={18} /> Tendencias (última nota vs. tu promedio anterior)
                </h3>
              </div>
              <div style={{ overflowX: 'auto' }}>
                <table className="table">
                  <thead>
                    <tr>
                      <th>Categoría</th>
                      <th>Reps</th>
                      <th>Ult. nota</th>
                      <th>Prom. previo</th>
                      <th>Δ</th>
                    </tr>
                  </thead>
                  <tbody>
                    {cats.map((c) => (
                      <tr key={c.key}>
                        <td>{c.name}</td>
                        <td>{c.appearances}</td>
                        <td>{c.last !== null ? Math.round(c.last) : '—'}</td>
                        <td>{c.prevAvg !== null ? Math.round(c.prevAvg) : '—'}</td>
                        <td>
                          <DeltaPill delta={c.delta} />
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              {weakestCat && (
                <p className="progress-note" style={{ marginTop: 12 }}>
                  Prioridad: <b style={{ color: 'var(--orange)' }}>{weakestCat.name}</b> (
                  {Math.round(weakestCat.last ?? 0)}). Es el área que te está costando más; los
                  consejos del último análisis apuntan ahí.
                </p>
              )}
            </div>
          </div>

          {last?.analysis && (last.analysis.progreso.mejoras.length > 0 || last.analysis.progreso.regresiones.length > 0) && (
            <div className="card">
              <div className="card-title">
                <h3>
                  <Wand2 size={18} /> ¿Qué dice el Maestro de tu progreso?
                </h3>
                <span className="pill pill-dim">{shortDate(last.createdAt)}</span>
              </div>
              <div className="grid grid-2">
                <div>
                  <h4 style={{ color: 'var(--good)', marginBottom: 8, fontSize: 13 }}>
                    Mejoras detectadas
                  </h4>
                  <ul className="check-list">
                    {last.analysis.progreso.mejoras.map((m, i) => (
                      <li key={i}>
                        <ArrowUpRight className="ic" size={15} style={{ color: 'var(--good)' }} />
                        <span>{m}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div>
                  <h4 style={{ color: 'var(--bad)', marginBottom: 8, fontSize: 13 }}>
                    Lo que sigue igual o empeoró
                  </h4>
                  <ul className="check-list">
                    {last.analysis.progreso.regresiones.map((r, i) => (
                      <li key={i}>
                        <ArrowDownRight className="ic" size={15} style={{ color: 'var(--bad)' }} />
                        <span>{r}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  )
}