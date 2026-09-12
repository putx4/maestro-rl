import { FileUp, History as HistoryIcon, LineChart, Radar, Settings } from 'lucide-react'
import { useAppStore } from '../store'
import type { PageId } from '../types'

const NAV: { id: PageId; label: string; icon: typeof Radar }[] = [
  { id: 'dashboard', label: 'Inicio', icon: Radar },
  { id: 'analyze', label: 'Analizar replay', icon: FileUp },
  { id: 'history', label: 'Historial', icon: HistoryIcon },
  { id: 'progress', label: 'Progreso', icon: LineChart },
  { id: 'settings', label: 'Ajustes', icon: Settings },
]

export function Sidebar() {
  const page = useAppStore((s) => s.page)
  const setPage = useAppStore((s) => s.setPage)
  const error = useAppStore((s) => s.error)
  const settingsLoaded = useAppStore((s) => s.settingsLoaded)

  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-logo">
          <Radar size={24} strokeWidth={2.2} />
        </div>
        <div>
          <div className="brand-name">
            Maestro <b>RL</b>
          </div>
          <div className="brand-sub">Coach de Rocket League</div>
        </div>
      </div>

      {NAV.map(({ id, label, icon: Icon }) => (
        <div
          key={id}
          className={`nav-item${page === id ? ' active' : ''}`}
          onClick={() => setPage(id)}
        >
          <Icon size={18} />
          {label}
        </div>
      ))}

      <div className="sidebar-foot">
        <span className={`server-dot${settingsLoaded && !error ? ' online' : ' offline'}`} />
        {settingsLoaded && !error
          ? 'opencode local'
          : error
            ? 'sin conectar'
            : 'cargando…'}
      </div>
    </aside>
  )
}