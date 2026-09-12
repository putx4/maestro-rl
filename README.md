# Maestro RL 🎮

![Licencia GPL-3.0](https://img.shields.io/badge/licencia-GPL--3.0-blue)

Tu **coach personal de Rocket League** con IA, 100% local.

Le pasás una replay (`.replay`), el Maestro la analiza con **opencode** corriendo en tu
máquina y te devuelve:

- **Nota general** y por categorías (Mecánicas, Defensa, Ataque, Rotaciones, Lectura, Boost, Mental)
- **Aciertos** concretos de tu partido
- **Errores** con contexto (revisa la cronología de goles)
- **Drills de entrenamiento** priorizados para tus puntos flacos
- **Progreso**: aprende de cada replay que le das y te dice en qué mejoraste (y en qué no)

## Stack

- **Frontend:** React 19 + TypeScript + Vite + Zustand + Recharts
- **Backend:** Tauri 2 (Rust) — parseo de replays con [`boxcars`](https://crates.io/crates/boxcars)
- **Análisis:** `opencode serve` en `http://127.0.0.1:4096` (sesión por análisis, se borra al terminar)
- **Datos:** `history.json` en la carpeta de datos de la app (por defecto
  `%APPDATA%\com.lacuca.maestrorl`)

## Requisitos

1. [Node.js](https://nodejs.org) + [Rust](https://rustup.rs) (stable)
2. [opencode](https://opencode.ai) instalado y disponible en la terminal
3. Replays de Rocket League (Están en `...\My Games\Rocket League\TAGame\Demos`)

## Correr en desarrollo

```bash
# Terminal 1 — servidor local de IA
opencode serve

# Terminal 2
npm install
npm run tauri dev
```

## Build / instalador

```bash
npm run tauri build   # genera el instalador en src-tauri/target/release/bundle
```

## Uso

1. **Ajustes** → configurá tu nick (p. ej. `lacuca3000`), rango y verificá la URL de opencode
   (Probar conexión).
2. **Analizar replay** → elegí el `.replay`, revisá lo que se leyó del partido y pulsá
   *Analizar con el Maestro*.
3. **Historial** guarda cada análisis (podés re-analizar o borrar).
4. **Progreso** grafica tu evolución por categoría y muestra las tendencias del historial.

> El análisis corre local: nada sale de tu PC. El Maestro usa los últimos análisis (configurables
> en *Memoria*) para comparar tu evolución en cada replay nueva.

## Prueba end-to-end sin la GUI

El backend tiene un binario de prueba que recorre todo el pipeline (parseo → prompt → opencode → análisis JSON) con una replay real:

```bash
opencode serve
cargo run --example e2e -- "ruta\a\partida.replay" "lacuca3000" http://127.0.0.1:4096
```

> Las stats de jugadores (goles, asistencias, salvadas, tiros, puntos, demoliciones, boost) se
> leen del **network data** (atributos `TAGame.PRI_TA:*`), no del header. Por eso el parser no
> usa `never_parse_network_data()`.

## Estructura

```
src/                    # Frontend (React)
  pages/                # Dashboard, Analyze, History, Progress, Settings
  components/           # UI (resultado del análisis, tabla de jugadores, layout)
  lib/                  # API (invoke tauri + dialog), formato
src-tauri/src/          # Backend (Rust)
  replay.rs             # Parser boxcars + extractor defensivo de stats
  coach.rs              # Cliente opencode + prompt + parser de análisis JSON
  store.rs              # Persistencia (history.json)
  commands.rs           # Comandos Tauri
  models.rs             # Tipos
```