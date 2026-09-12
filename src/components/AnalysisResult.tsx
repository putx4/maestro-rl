import { ArrowDownRight, ArrowUpRight, Check, Dumbbell, ShieldX, Sparkles, X, Zap } from 'lucide-react'
import type { Analysis } from '../types'
import { ScoreRing } from './ui'

function barClass(nota: number) {
  if (nota < 55) return 'cat-bar-fill low'
  if (nota < 75) return 'cat-bar-fill mid'
  return 'cat-bar-fill'
}

export function AnalysisResult({ analysis }: { analysis: Analysis }) {
  return (
    <div className="grid" style={{ gap: 16 }}>
      <div className="card">
        <div className="grid grid-2" style={{ alignItems: 'center', gap: 26 }}>
          <div style={{ justifySelf: 'center' }}>
            <ScoreRing score={analysis.puntuacion} label="Nota del Maestro" />
          </div>
          <div>
            <h3 style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <Sparkles size={18} style={{ color: 'var(--orange)' }} />
              Resumen del partido
            </h3>
            <p style={{ color: 'var(--text-dim)' }}>{analysis.resumen}</p>

            {(analysis.progreso.mejoras.length > 0 ||
              analysis.progreso.regresiones.length > 0) && (
              <div className="chip-line" style={{ marginTop: 14 }}>
                {analysis.progreso.mejoras.map((m, i) => (
                  <span key={`up-${i}`} className="pill pill-good">
                    <ArrowUpRight size={14} /> {m}
                  </span>
                ))}
                {analysis.progreso.regresiones.map((r, i) => (
                  <span key={`dn-${i}`} className="pill pill-bad">
                    <ArrowDownRight size={14} /> {r}
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {analysis.categorias.length > 0 && (
        <div className="card">
          <div className="card-title">
            <h3>
              <Zap size={18} /> Categorías
            </h3>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            {analysis.categorias.map((c) => (
              <div className="cat-row" key={c.key}>
                <span className="cat-name" title={c.comentario || c.name}>
                  {c.name}
                </span>
                <div className="cat-bar-track">
                  <div
                    className={barClass(c.nota)}
                    style={{ width: `${Math.max(2, Math.min(100, c.nota))}%` }}
                  />
                </div>
                <span className="cat-note">{Math.round(c.nota)}</span>
                {c.comentario && (
                  <span className="progress-note" style={{ gridColumn: '1 / -1', textAlign: 'right' }}>
                    {c.comentario}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="grid grid-2">
        {analysis.aciertos.length > 0 && (
          <div className="card">
            <div className="card-title">
              <h3 style={{ color: 'var(--good)' }}>
                <Check size={18} /> Lo que hiciste bien
              </h3>
            </div>
            <ul className="check-list">
              {analysis.aciertos.map((a, i) => (
                <li key={i}>
                  <Check className="ic good-ic" size={16} />
                  <span>{a}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {analysis.errores.length > 0 && (
          <div className="card">
            <div className="card-title">
              <h3 style={{ color: 'var(--bad)' }}>
                <X size={18} /> Errores a corregir
              </h3>
            </div>
            <ul className="check-list">
              {analysis.errores.map((e, i) => (
                <li key={i}>
                  <X className="ic bad-ic" size={16} />
                  <span>{e}</span>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {analysis.consejos.length > 0 && (
        <div className="card">
          <div className="card-title">
            <h3 style={{ color: 'var(--blue)' }}>
              <Zap size={18} /> Orden de entrenamiento (de más a menos importante)
            </h3>
          </div>
          <div className="grid" style={{ gap: 10 }}>
            {analysis.consejos.map((c, i) => (
              <div className="consejo" key={i}>
                <h4>
                  {i + 1}. {c.titulo}
                </h4>
                {c.explicacion && <div className="explicacion">{c.explicacion}</div>}
                {c.drill && (
                  <div className="drill">
                    <Dumbbell size={15} />
                    <span>
                      <b>Drill:</b> {c.drill}
                    </span>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {analysis.resumen && analysis.consejos.length === 0 && analysis.categorias.length === 0 && (
        <div className="card">
          <div className="card-title">
            <h3>
              <ShieldX size={18} />
              Respuesta cruda del Maestro
            </h3>
          </div>
          <div className="raw-box">{analysis.resumen}</div>
        </div>
      )}
    </div>
  )
}