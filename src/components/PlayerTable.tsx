import type { PlayerStat } from '../types'

export function PlayerTable({
  players,
  nick,
}: {
  players: PlayerStat[]
  nick: string
}) {
  return (
    <div style={{ overflowX: 'auto' }}>
      <table className="table">
        <thead>
          <tr>
            <th>Jugador</th>
            <th>Eq</th>
            <th>Pts</th>
            <th>G</th>
            <th>A</th>
            <th>S</th>
            <th>T</th>
            <th>Dem</th>
            <th>Boost</th>
            <th>AutoG</th>
          </tr>
        </thead>
        <tbody>
          {players.map((p) => {
            const me = p.name.trim().toLowerCase() === nick.trim().toLowerCase()
            return (
              <tr key={p.name} className={me ? 'me' : undefined}>
                <td>
                  {p.name}
                  {me && <span className="pill pill-blue" style={{ marginLeft: 8 }}>TÚ</span>}
                </td>
                <td>{p.team >= 0 ? p.team : '—'}</td>
                <td>{p.matchScore}</td>
                <td>{p.goals}</td>
                <td>{p.assists}</td>
                <td>{p.saves}</td>
                <td>{p.shots}</td>
                <td>{p.demolishes}</td>
                <td
                  title={
                    p.boostPickups > 0
                      ? `${p.boostSmallPads} pads chicos, ${p.boostBigPads} pads grandes`
                      : undefined
                  }
                >
                  {p.avgBoost > 0 ? (
                    <>
                      {Math.round(p.avgBoost)}%
                      {p.boostPickups > 0 && (
                        <div style={{ fontSize: 11, color: 'var(--text-dim)' }}>
                          {p.boostSmallPads}+{p.boostBigPads}
                        </div>
                      )}
                    </>
                  ) : (
                    p.boostPickups
                  )}
                </td>
                <td>{p.ownGoals > 0 ? p.ownGoals : '0'}</td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}