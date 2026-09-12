export function shortDate(iso: string): string {
  try {
    const d = new Date(iso)
    if (Number.isNaN(d.getTime())) return iso.slice(0, 10)
    return d.toLocaleDateString('es-AR', { day: '2-digit', month: '2-digit' })
  } catch {
    return iso.slice(0, 10)
  }
}

export function fullDate(iso: string): string {
  try {
    const d = new Date(iso)
    if (Number.isNaN(d.getTime())) return iso
    return d.toLocaleString('es-AR', {
      day: '2-digit',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return iso
  }
}

export function durLabel(frames: number | null): string {
  if (!frames) return '—'
  const secs = frames / 30
  const min = Math.floor(secs / 60)
  const s = Math.round(secs % 60)
  return `${min}:${String(s).padStart(2, '0')}`
}