import { useEffect } from 'react'
import { Sidebar } from './components/Sidebar'
import { TopBar } from './components/TopBar'
import { useAppStore } from './store'
import { Dashboard } from './pages/Dashboard'
import { Analyze } from './pages/Analyze'
import { History } from './pages/History'
import { Progress } from './pages/Progress'
import { Settings } from './pages/Settings'

export default function App() {
  const page = useAppStore((s) => s.page)
  const init = useAppStore((s) => s.init)

  useEffect(() => {
    init()
  }, [init])

  return (
    <div className="app">
      <Sidebar />
      <div className="main">
        <TopBar />
        <div className="content">
          {page === 'dashboard' && <Dashboard />}
          {page === 'analyze' && <Analyze />}
          {page === 'history' && <History />}
          {page === 'progress' && <Progress />}
          {page === 'settings' && <Settings />}
        </div>
      </div>
    </div>
  )
}