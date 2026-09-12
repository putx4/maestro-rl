import { AlertTriangle, Loader2 } from 'lucide-react'
import type { ReactNode } from 'react'

export function Stat({
  label,
  value,
  sub,
  className,
  icon,
}: {
  label: ReactNode
  value: ReactNode
  sub?: ReactNode
  className?: string
  icon?: ReactNode
}) {
  return (
    <div className={`card stat${className ? ` ${className}` : ''}`}>
      <span className="stat-label">
        {icon}
        {label}
      </span>
      <span className="stat-value">{value}</span>
      {sub && <span className="stat-sub">{sub}</span>}
    </div>
  )
}

export function ScoreRing({
  score,
  size = 128,
  label,
}: {
  score: number
  size?: number
  label?: string
}) {
  const r = (size - 14) / 2
  const c = 2 * Math.PI * r
  const clamped = Math.max(0, Math.min(100, score))
  const color = clamped >= 75 ? 'var(--good)' : clamped >= 55 ? 'var(--warn)' : 'var(--bad)'

  return (
    <div className="ring-wrap">
      <svg width={size} height={size}>
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke="rgba(255,255,255,0.08)"
          strokeWidth="10"
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke={color}
          strokeWidth="10"
          strokeLinecap="round"
          strokeDasharray={c}
          strokeDashoffset={c * (1 - clamped / 100)}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          style={{ transition: 'stroke-dashoffset 0.8s ease' }}
        />
        <text
          x="50%"
          y="54%"
          dominantBaseline="middle"
          textAnchor="middle"
          fontSize={size / 4.4}
          fontWeight={800}
          fill="#eef2fa"
          style={{ fontVariantNumeric: 'tabular-nums' }}
        >
          {Math.round(clamped)}
        </text>
      </svg>
      {label && <span className="ring-label">{label}</span>}
    </div>
  )
}

export function EmptyState({
  icon,
  title,
  subtitle,
  children,
}: {
  icon: ReactNode
  title: string
  subtitle?: string
  children?: ReactNode
}) {
  return (
    <div className="empty">
      {icon}
      <h3>{title}</h3>
      {subtitle && <p>{subtitle}</p>}
      {children}
    </div>
  )
}

export function ErrorBanner({ message }: { message: string }) {
  return (
    <div className="error-banner">
      <AlertTriangle size={17} />
      <div>{message}</div>
    </div>
  )
}

export function Loading({ label = 'Cargando…' }: { label?: string }) {
  return (
    <div className="loading-screen">
      <Loader2 className="spinner" size={20} />
      {label}
    </div>
  )
}