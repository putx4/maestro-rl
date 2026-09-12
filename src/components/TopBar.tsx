import { useAppStore } from '../store'
import type { PageId } from '../types'

const TITLES: Record<PageId, { crumb: string; title: string }> = {
  dashboard: { crumb: 'Panel', title: 'Tu camino a Gran Campeón' },
  analyze: { crumb: 'Coach', title: 'Analizar replay' },
  history: { crumb: 'Coach', title: 'Historial de replays' },
  progress: { crumb: 'Coach', title: 'Progreso y evolución' },
  settings: { crumb: 'Sistema', title: 'Ajustes' },
}

export function TopBar() {
  const page = useAppStore((s) => s.page)
  const settings = useAppStore((s) => s.settings)
  const t = TITLES[page]

  return (
    <header className="topbar">
      <div className="topbar-title">
        <span className="crumb">{t.crumb}</span>
        <h1>{t.title}</h1>
      </div>
      <div className="topbar-right">
        <span className="rank pill pill-orange" title="Rango configurado en Ajustes">
          {settings.rank}
        </span>
        <span className="nick-chip">
          {settings.nick}
        </span>
      </div>
    </header>
  )
}