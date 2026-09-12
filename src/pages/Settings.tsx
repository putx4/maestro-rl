import { useEffect, useState } from 'react'
import { Check, Loader2, Plug, Save, Server, Trash2 } from 'lucide-react'
import { api } from '../lib/api'
import { useAppStore, stringifyErr } from '../store'
import type { CoachSettings } from '../types'
import { ErrorBanner, Loading } from '../components/ui'

export function Settings() {
  const { settings, saveSettings, clearHistory, entries, loaded } = useAppStore()
  const [form, setForm] = useState<CoachSettings>(settings)
  const [saving, setSaving] = useState(false)
  const [testing, setTesting] = useState(false)
  const [clearing, setClearing] = useState(false)
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null)
  const [err, setErr] = useState<string | null>(null)

  useEffect(() => {
    if (loaded) setForm(settings)
  }, [loaded, settings])

  if (!loaded) return <Loading />

  const set = <K extends keyof CoachSettings>(key: K, value: CoachSettings[K]) =>
    setForm((f) => ({ ...f, [key]: value }))

  const save = async () => {
    setSaving(true)
    setErr(null)
    setMsg(null)
    try {
      await saveSettings(form)
      setMsg({ ok: true, text: 'Ajustes guardados.' })
    } catch (e) {
      setErr(stringifyErr(e))
    } finally {
      setSaving(false)
    }
  }

  const test = async () => {
    setTesting(true)
    setErr(null)
    setMsg(null)
    try {
      const r = await api.testConnection()
      setMsg({ ok: r.ok, text: r.message })
    } catch (e) {
      setErr(stringifyErr(e))
    } finally {
      setTesting(false)
    }
  }

  const clearAll = async () => {
    if (!confirm(`¿Borrar los ${entries.length} análisis del historial? Esta acción no se puede deshacer.`)) {
      return
    }
    setClearing(true)
    setErr(null)
    try {
      await clearHistory()
      setMsg({ ok: true, text: 'Historial borrado.' })
    } catch (e) {
      setErr(stringifyErr(e))
    } finally {
      setClearing(false)
    }
  }

  return (
    <div className="grid" style={{ gap: 18, maxWidth: 760 }}>
      {err && <ErrorBanner message={err} />}
      {msg && (
        <div className={`banner${msg.ok ? ' success-banner' : ''}`}>
          <Check size={16} />
          <span>{msg.text}</span>
        </div>
      )}

      <div className="card">
        <div className="card-title">
          <h3>
            <Server size={18} /> Servidor opencode
          </h3>
        </div>
        <div className="grid grid-2">
          <div className="field">
            <label>URL del servidor local</label>
            <input
              value={form.serverUrl}
              onChange={(e) => set('serverUrl', e.target.value)}
              spellCheck={false}
              placeholder="http://127.0.0.1:4096"
            />
            <span className="hint">
              Lo levantás con <span className="kbd">opencode serve</span> en una terminal. Todo corre
              en tu máquina.
            </span>
          </div>
          <div className="field">
            <label>Timeout del análisis (segundos)</label>
            <input
              type="number"
              min={60}
              max={1200}
              value={form.timeoutSecs}
              onChange={(e) => set('timeoutSecs', Number(e.target.value))}
            />
            <span className="hint">Si el modelo tarda más, se corta con error.</span>
          </div>
        </div>
        <div style={{ display: 'flex', gap: 10, marginTop: 16 }}>
          <button className="btn btn-ghost" onClick={test} disabled={testing}>
            {testing ? <Loader2 className="spinner" size={16} /> : <Plug size={16} />}
            Probar conexión
          </button>
        </div>
      </div>

      <div className="card">
        <div className="card-title">
          <h3>
            <Check size={18} /> Tu perfil de jugador
          </h3>
        </div>
        <div className="grid grid-2">
          <div className="field">
            <label>Nick en Rocket League</label>
            <input
              value={form.nick}
              onChange={(e) => set('nick', e.target.value)}
              spellCheck={false}
            />
            <span className="hint">El Maestro solo analiza a este jugador en cada replay.</span>
          </div>
          <div className="field">
            <label>Rango actual</label>
            <input value={form.rank} onChange={(e) => set('rank', e.target.value)} />
          </div>
          <div className="field">
            <label>Modos que jugás</label>
            <input value={form.modes} onChange={(e) => set('modes', e.target.value)} />
          </div>
          <div className="field">
            <label>Idioma del análisis</label>
            <select value={form.language} onChange={(e) => set('language', e.target.value)}>
              <option value="es">Español</option>
              <option value="en">English</option>
            </select>
          </div>
          <div className="field">
            <label>Memoria del Maestro (nº de replays que recuerda)</label>
            <input
              type="number"
              min={1}
              max={50}
              value={form.memoryDepth}
              onChange={(e) => set('memoryDepth', Number(e.target.value))}
            />
            <span className="hint">
              Con cada análisis nuevo, compara con estos últimos para decirte en qué mejoraste.
            </span>
          </div>
        </div>
        <div style={{ display: 'flex', gap: 10, marginTop: 16 }}>
          <button className="btn btn-primary" onClick={save} disabled={saving}>
            {saving ? <Loader2 className="spinner" size={16} /> : <Save size={16} />}
            Guardar ajustes
          </button>
        </div>
      </div>

      <div className="card" style={{ borderColor: 'rgba(255,95,102,0.25)' }}>
        <div className="card-title">
          <h3 style={{ color: 'var(--bad)' }}>
            <Trash2 size={18} /> Zona de peligro
          </h3>
        </div>
        <p className="progress-note" style={{ marginBottom: 12 }}>
          Borra todo tu historial de análisis. El Maestro volvería a empezar de cero (sin saber tu
          progreso). Tu nick y ajustes se conservan.
        </p>
        <button className="btn btn-danger" onClick={clearAll} disabled={clearing || entries.length === 0}>
          {clearing ? <Loader2 className="spinner" size={15} /> : <Trash2 size={15} />}
          Borrar historial ({entries.length})
        </button>
      </div>
    </div>
  )
}