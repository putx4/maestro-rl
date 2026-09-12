use crate::error::{AppError, Result};
use crate::models::*;
use serde_json::{json, Value};

pub struct CoachClient {
    client: reqwest::Client,
    base: String,
    timeout: std::time::Duration,
    session_title: String,
}

impl CoachClient {
    const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

    pub fn new(server_url: &str, timeout_secs: u64) -> Self {
        Self {
            client: reqwest::Client::new(),
            base: server_url.trim_end_matches('/').to_string(),
            timeout: std::time::Duration::from_secs(timeout_secs.max(60)),
            session_title: "maestro-rl-coach".to_string(),
        }
    }

    pub async fn create_session(&self) -> Result<String> {
        let body = json!({ "title": self.session_title });
        let resp = self
            .client
            .post(format!("{}/session", self.base))
            .json(&body)
            .timeout(Self::CONNECT_TIMEOUT)
            .send()
            .await
            .map_err(|e| {
                AppError::Coach(format!(
                    "No se pudo conectar con opencode en {} (¿está corriendo `opencode serve`?). Detalle: {}",
                    self.base, e
                ))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Coach(format!(
                "Falló al crear la sesión con opencode (HTTP {}): {}",
                status, text
            )));
        }

        let v: Value = resp.json().await.map_err(|e| {
            AppError::Coach(format!("Respuesta inválida de opencode al crear sesión: {e}"))
        })?;

        v["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Coach("opencode no devolvió un id de sesión".into()))
    }

    pub async fn send_message(&self, session_id: &str, prompt: &str) -> Result<String> {
        let endpoint = format!("{}/session/{}/message", self.base, session_id);
        let body = json!({ "parts": [{ "type": "text", "text": prompt }] });

        let resp = self
            .client
            .post(&endpoint)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::Coach(format!(
                        "opencode no respondió en {}s. ¿El modelo está tardando mucho? Revisá en Ajustes el timeout.",
                        self.timeout.as_secs()
                    ))
                } else {
                    AppError::Coach(format!("Error de red con el servidor opencode: {e}"))
                }
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Coach(format!(
                "El mensaje falló (HTTP {}): {}",
                status, text
            )));
        }

        let v: Value = resp.json().await.map_err(|e| {
            AppError::Coach(format!("Respuesta inválida de opencode: {e}"))
        })?;

        let mut collected = String::new();
        if let Some(parts) = v["parts"].as_array() {
            for part in parts {
                let is_text = part["type"]
                    .as_str()
                    .map(|t| t.eq_ignore_ascii_case("text"))
                    .unwrap_or(false);
                if is_text {
                    if let Some(t) = part["text"].as_str() {
                        collected.push_str(t);
                        collected.push('\n');
                    }
                }
            }
        }

        let out = collected.trim();
        if out.is_empty() {
            if let Some(err) = v["info"]["error"].as_str() {
                return Err(AppError::Coach(format!("opencode respondió con un error: {err}")));
            }
            return Err(AppError::Coach(
                "opencode no devolvió texto en la respuesta".into(),
            ));
        }
        Ok(out.to_string())
    }

    pub async fn delete_session(&self, session_id: &str) {
        let _ = self
            .client
            .delete(format!("{}/session/{}", self.base, session_id))
            .timeout(Self::CONNECT_TIMEOUT)
            .send()
            .await;
    }

    /// Runs a full coaching pass: create session, send prompt, parse, cleanup.
    pub async fn analyze(&self, prompt: &str) -> Result<String> {
        let session_id = self.create_session().await?;
        let res = self.send_message(&session_id, prompt).await;
        self.delete_session(&session_id).await;
        res
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{}…", cut)
    }
}

fn players_table(players: &[PlayerStat], nick: &str) -> String {
    players
        .iter()
        .map(|p| {
            let marker = if p.name.eq_ignore_ascii_case(nick.trim()) {
                " ← ERES TÚ"
            } else {
                ""
            };
            let boost = if p.avg_boost > 0.0 {
                format!(
                    "{:.0}% ({}+{} pads)",
                    p.avg_boost, p.boost_small_pads, p.boost_big_pads
                )
            } else {
                format!("{} pads", p.boost_pickups)
            };
            format!(
                "- {p:<24} equipo {team} | puntos {puntos} | G {g} A {a} S {s} T {t} | demolish {d} | boost {boost} | clears {clr} denials {den} steals {stl} | daño {dmg}{marker}",
                p = p.name,
                team = p.team,
                puntos = p.match_score,
                g = p.goals,
                a = p.assists,
                s = p.saves,
                t = p.shots,
                d = p.demolishes,
                clr = p.clears,
                den = p.denials,
                stl = p.steals,
                dmg = p.breakout_damage,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn goals_timeline(goals: &[GoalEvent], max: usize) -> String {
    let take = goals.len().min(max.max(1));
    goals
        .iter()
        .take(take)
        .map(|g| {
            let own = if g.own_goal { " (EN PROPIA)" } else { "" };
            format!(
                "minute ~{:.0} | {} → gol equipo {}",
                (g.frame as f64 / 1800.0),
                g.player_name,
                g.team
            ) + own
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn build_memory_digest(entries: &[AnalysisEntry], nick: &str, limit: usize) -> String {
    let analyzed: Vec<&AnalysisEntry> = entries
        .iter()
        .filter(|e| {
            e.analysis.is_some()
                && (nick.is_empty()
                    || e.summary.players.iter().any(|p| {
                        p.name.eq_ignore_ascii_case(nick.trim())
                    }))
        })
        .rev()
        .take(limit)
        .collect();

    if analyzed.is_empty() {
        return "No hay análisis previos guardados.".to_string();
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "({} análisis previos, se muestran los {} más recientes)",
        analyzed.len(),
        limit
    ));

    for e in analyzed {
        let a = e.analysis.as_ref().expect("filtered");
        let me = e.summary.players.iter().find(|p| {
            p.name.eq_ignore_ascii_case(nick.trim())
        });
        let result = match (&e.summary.team_scores[..], me) {
            ([t0, t1], Some(m)) if t0 != t1 => {
                let my_team = m.team;
                let won = if my_team == 0 {
                    t0 > t1
                } else {
                    t1 > t0
                };
                if won {
                    format!("VICTORIA {}:{}", t0, t1)
                } else {
                    format!("DERROTA {}:{}", t0, t1)
                }
            }
            ([t0, t1], _) => format!("{}:{}", t0, t1),
            _ => "?"[..].to_string(),
        };
        let stats = me.map(|m| {
            format!(
                "tuyas: {}pts G{} A{} S{} T{} Dem{}",
                m.match_score, m.goals, m.assists, m.saves, m.shots, m.demolishes,
            )
        });
        let cats = a
            .categorias
            .iter()
            .map(|c| format!("{} {:.0}", c.name, c.nota))
            .collect::<Vec<_>>()
            .join(", ");
        let errores: Vec<String> = a.errores.iter().take(2).cloned().collect();
        let tareas: Vec<String> = a.consejos.iter().take(2).map(|c| c.titulo.clone()).collect();
        lines.push(format!(
            "· {} | {} | {} {} | nota {:.0} | {} | fallos: {} | a trabajar: {}",
            e.created_at.chars().take(10).collect::<String>(),
            e.summary
                .map_name
                .clone()
                .unwrap_or_else(|| "?".to_string()),
            result,
            stats.as_deref().unwrap_or(""),
            a.puntuacion,
            cats,
            if errores.is_empty() {
                "—".to_string()
            } else {
                errores.join("; ")
            },
            if tareas.is_empty() {
                "—".to_string()
            } else {
                tareas.join("; ")
            },
        ));
    }

    lines.join("\n")
}

pub fn build_prompt(
    settings: &CoachSettings,
    summary: &ReplaySummary,
    raw_properties: &Value,
    memory_digest: &str,
) -> String {
    let raw_json = truncate(&serde_json::to_string(raw_properties).unwrap_or_default(), 6000);
    let duration = summary
        .duration_secs()
        .map(|s| format!("{:.0} min", s / 60.0))
        .unwrap_or_else(|| "?".to_string());

    let lang = match settings.language.as_str() {
        "en" => "English",
        _ => "Español",
    };

    format!(
"Eres el Maestro de Rocket League, un coach personal y exigente pero constructivo para el jugador \"{nick}\" (rango aproximado: {rank}, modos: {modes}).

Debes responder SIEMPRE en {language} y SOLO como JSON válido (sin bloques de código, sin texto extra alrededor del JSON).

Tarea: analizar ÚNICAMENTE el desempeño de \"{nick}\" en esta replay. Ignorá a los demás jugadores salvo como contexto. Sé concreto, citando números reales de sus estadísticas y el contexto del partido. Decí qué hizo bien, qué hizo mal y exactamente qué practicar para mejorar.

=== PARTIDO ===
- Mapa: {map}
- Modo: {mode}
- Tipo: {match_type}
- Duración: {duration}
- Marcador final: {score}
- Team size: {team_size}

=== TU JUGADOR (a analizar) ===
{me}

=== TODOS LOS JUGADORES ===
{players}

=== GOLES (cronología) ===
{goals}

=== DATOS ADICIONALES DEL REPLAY (JSON) ===
{raw}

=== MEMORIA DEL MAESTRO (análisis previos, para ver progreso) ===
{memory}

Formato EXACTO de respuesta (JSON):
{{
  \"resumen\": \"párrafo corto del partido y tu desempeño\",
  \"puntuacion\": 0-100,
  \"categorias\": [
    {{ \"nombre\": \"Mecánicas\", \"nota\": 0-100, \"comentario\": \"1-2 frases\" }},
    {{ \"nombre\": \"Defensa\", \"nota\": 0-100, \"comentario\": \"...\" }},
    {{ \"nombre\": \"Ataque\", \"nota\": 0-100, \"comentario\": \"...\" }},
    {{ \"nombre\": \"Rotaciones\", \"nota\": 0-100, \"comentario\": \"...\" }},
    {{ \"nombre\": \"Lectura de juego\", \"nota\": 0-100, \"comentario\": \"...\" }},
    {{ \"nombre\": \"Boost\", \"nota\": 0-100, \"comentario\": \"...\" }},
    {{ \"nombre\": \"Mental\", \"nota\": 0-100, \"comentario\": \"...\" }}
  ],
  \"aciertos\": [\"3-5 cosas que hizo bien, concretas\"],
  \"errores\": [\"3-5 fallos concretos con la situación (revisá la cronología de goles si aplica)\"],
  \"consejos\": [
    {{ \"titulo\": \"Nombre corto del hábito a entrenar\", \"explicacion\": \"por qué y cómo\", \"drill\": \"ejercicio concreto (tiempo, objetivo) compatible con los mapas gratuitos habituales o packs de entrenamiento\" }}
  ],
  \"progreso\": {{ \"mejoras\": [\"qué viene mejorando vs la memoria\"], \"regresiones\": [\"qué empeoró o sigue igual (USA la memoria; si no hay análisis previos, pon \\\"—\\\"\")] }}
}}
",
        nick = settings.nick,
        rank = settings.rank,
        modes = settings.modes,
        language = lang,
        map = summary.map_name.clone().unwrap_or_else(|| "?".into()),
        mode = summary.game_mode.clone().unwrap_or_else(|| "?".into()),
        match_type = summary.match_type.clone().unwrap_or_else(|| "?".into()),
        duration = duration,
        score = if summary.team_scores.len() == 2 {
            format!("{} - {}", summary.team_scores[0], summary.team_scores[1])
        } else {
            "?".to_string()
        },
        team_size = summary
            .team_size
            .map(|s| s.to_string())
            .unwrap_or_else(|| "?".into()),
        me = summary
            .players
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(settings.nick.trim()))
            .map(|p| players_table(std::slice::from_ref(p), &settings.nick))
            .unwrap_or_else(|| {
                format!(
                    "⚠ No apareció \"{}\" en esta replay (los jugadores del archivo: {}). Analizalo igual con los datos del partido.",
                    settings.nick,
                    summary
                        .players
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }),
        players = players_table(&summary.players, &settings.nick),
        goals = goals_timeline(&summary.goals, 14),
        raw = raw_json,
        memory = memory_digest,
    )
}

fn strip_fence(s: &str) -> &str {
    let t = s.trim();
    for prefix in ["```json", "```jsonl", "```"] {
        if let Some(rest) = t.strip_prefix(prefix) {
            return rest.trim_end_matches("```").trim();
        }
    }
    t
}

pub fn parse_analysis(raw: &str) -> Analysis {
    let cleaned = strip_fence(raw);
    let json = extract_json_obj(cleaned).unwrap_or_else(|| cleaned.to_string());

    let mut analysis = match serde_json::from_str::<Value>(&json) {
        Ok(v) => from_value(&v),
        Err(_) => {
            // Last resort: keep everything as a plain summary.
            let mut a = Analysis::default();
            a.resumen = cleaned.to_string();
            a
        }
    };

    if analysis.puntuacion <= 0.0 && !analysis.categorias.is_empty() {
        let avg = analysis
            .categorias
            .iter()
            .map(|c| c.nota)
            .sum::<f64>()
            / analysis.categorias.len() as f64;
        analysis.puntuacion = round1(avg);
    }

    analysis
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn extract_json_obj(s: &str) -> Option<String> {
    let start = s.find('{')?;
    let end = s.rfind('}')?;
    if end > start {
        Some(s[start..=end].to_string())
    } else {
        None
    }
}

fn from_value(v: &Value) -> Analysis {
    let mut a = Analysis::default();

    if let Some(t) = first_str(v, &["resumen", "summary", "analisis"]) {
        a.resumen = t;
    }

    if let Some(n) = first_f64(v, &["puntuacion", "nota_general", "score", "puntaje"]) {
        a.puntuacion = clamp(n, 0.0, 100.0);
    }

    if let Some(arr) = first_array(v, &["categorias", "categories", "puntuaciones"]) {
        for item in arr {
            let mut name = first_str(item, &["nombre", "name", "categoria", "category"])
                .unwrap_or_else(|| "General".to_string());
            if name.eq_ignore_ascii_case("General") {
                name = "General".to_string();
            }
            let nota = first_f64(item, &["nota", "score", "puntuacion", "rating", "value"])
                .map(|n| clamp(n, 0.0, 100.0))
                .unwrap_or(0.0);
            let comentario = first_str(item, &["comentario", "comment", "coment"])
                .unwrap_or_default();
            let (key, display) = canon_category(&name);
            a.categorias.push(CategoryScore {
                key,
                name: display,
                nota: round1(nota),
                comentario,
            });
        }
    }

    a.aciertos = first_strings(
        v,
        &["aciertos", "positivas", "positives", "cosas_bien", "good", "strengths"],
    );

    a.errores = first_strings(
        v,
        &["errores", "fallos", "mistakes", "negativas", "weaknesses", "points_to_improve"],
    );

    if let Some(arr) = first_array(v, &["consejos", "tips", "advice", "drills", "recomendaciones"]) {
        for item in arr {
            let titulo = first_str(item, &["titulo", "title", "nombre", "name", "tarea"])
                .unwrap_or_else(|| "Consejo".to_string());
            let explicacion = first_str(item, &["explicacion", "explication", "explanation", "detalle"])
                .unwrap_or_default();
            let drill = first_str(item, &["drill", "ejercicio", "practice", "rutina"])
                .unwrap_or_default();
            a.consejos.push(Consejo {
                titulo,
                explicacion,
                drill,
            });
        }
    }

    if let Some(p) = v.get("progreso").or_else(|| v.get("progress")) {
        a.progreso.mejoras = strings_from(p, &["mejoras", "improvements", "mejorado"]);
        a.progreso.regresiones = strings_from(p, &["regresiones", "regressions", "empeorado"]);
    }

    a
}

fn first_str<'a>(v: &'a Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(Value::as_str) {
            return Some(s.to_string());
        }
    }
    None
}

fn first_f64(v: &Value, keys: &[&str]) -> Option<f64> {
    for k in keys {
        if let Some(n) = v.get(*k).and_then(Value::as_f64) {
            return Some(n);
        }
    }
    None
}

fn first_array<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a Vec<Value>> {
    for k in keys {
        if let Some(arr) = v.get(*k).and_then(Value::as_array) {
            return Some(arr);
        }
    }
    None
}

fn first_strings(v: &Value, keys: &[&str]) -> Vec<String> {
    if let Some(arr) = first_array(v, keys) {
        arr.iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect()
    } else {
        Vec::new()
    }
}

fn strings_from(v: &Value, keys: &[&str]) -> Vec<String> {
    if let Some(arr) = first_array(v, keys) {
        arr.iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect()
    } else {
        Vec::new()
    }
}

fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

fn canon_category(name: &str) -> (String, String) {
    const ALIASES: &[(&str, &str, &str)] = &[
        ("mecanic", "Mecánicas", "mecanicas"),
        ("mechanics", "Mecánicas", "mecanicas"),
        ("aerial", "Mecánicas", "mecanicas"),
        ("defens", "Defensa", "defensa"),
        ("defense", "Defensa", "defensa"),
        ("ataque", "Ataque", "ataque"),
        ("attack", "Ataque", "ataque"),
        ("offens", "Ataque", "ataque"),
        ("posicion", "Posicionamiento", "posicionamiento"),
        ("positioning", "Posicionamiento", "posicionamiento"),
        ("rotaci", "Rotaciones", "rotaciones"),
        ("rotation", "Rotaciones", "rotaciones"),
        ("lectur", "Lectura de juego", "lectura"),
        ("reading", "Lectura de juego", "lectura"),
        ("awareness", "Lectura de juego", "lectura"),
        ("juego seguro", "Seguridad", "seguridad"),
        ("boost", "Boost", "boost"),
        ("mental", "Mental", "mental"),
        ("consisten", "Consistencia", "consistencia"),
        ("comunica", "Comunicación", "comunicacion"),
        ("communication", "Comunicación", "comunicacion"),
        ("velocidad", "Velocidad", "velocidad"),
        ("speed", "Velocidad", "velocidad"),
        ("decision", "Decisiones", "decisiones"),
        ("decisions", "Decisiones", "decisiones"),
        ("recuperaci", "Recuperación", "recuperacion"),
        ("recovery", "Recuperación", "recuperacion"),
        ("general", "General", "general"),
    ];

    let lower = name.to_lowercase();
    for (needle, display, key) in ALIASES {
        if lower.contains(needle) {
            return (key.to_string(), display.to_string());
        }
    }

    let key: String = lower
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .map(|c| {
            if c == ' ' {
                '-'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();
    let key = if key.is_empty() { "general".to_string() } else { key };
    (key, name.trim().to_string())
}