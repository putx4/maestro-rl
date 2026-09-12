use maestro_rl_lib::coach::{build_memory_digest, build_prompt, parse_analysis, CoachClient};
use maestro_rl_lib::models::CoachSettings;
use maestro_rl_lib::replay::parse_replay_file;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: e2e <replay.replay> [nick] [serverUrl]");
        std::process::exit(2);
    }
    let path = &args[1];
    let mut settings = CoachSettings::default();
    if let Some(nick) = args.get(2) {
        settings.nick = nick.clone();
    }
    if let Some(url) = args.get(3) {
        settings.server_url = url.clone();
    }
    let url = settings.server_url.clone();

    println!("== 1) PARSEO boxcars ==");
    let parsed = match parse_replay_file(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("FALLO al parsear: {e}");
            std::process::exit(1);
        }
    };
    println!("Mapa: {:?}", parsed.summary.map_name);
    println!("Modo: {:?} | tipo: {:?} | team_size: {:?}", parsed.summary.game_mode, parsed.summary.match_type, parsed.summary.team_size);
    println!("Marcador: {:?}", parsed.summary.team_scores);
    println!("Goles: {} | Duración: {:?}s", parsed.summary.goals.len(), parsed.summary.duration_secs());
    println!("Jugadores ({})", parsed.summary.players.len());
    for p in &parsed.summary.players {
        println!(
            "  - {:<24} eq {} | pts {} | G{} A{} S{} T{} | demolish {} | boost {}",
            p.name, p.team, p.match_score, p.goals, p.assists, p.saves, p.shots, p.demolishes, p.boost_pickups
        );
    }
    println!("Propiedades crudas: {} bytes", parsed.raw_properties.to_string().len());

    println!("\n== 2) PROMPT ==");
    let memory = build_memory_digest(&[], &settings.nick, settings.memory_depth);
    let prompt = build_prompt(&settings, &parsed.summary, &parsed.raw_properties, &memory);
    println!("Prompt: {} caracteres", prompt.chars().count());
    println!("Primeras líneas:\n{}", prompt.lines().take(6).collect::<Vec<_>>().join("\n"));

    println!("\n== 3) COACH (opencode {url}) ==");
    let client = CoachClient::new(&url, settings.timeout_secs);
    match client.analyze(&prompt).await {
        Ok(raw) => {
            println!("Respuesta cruda: {} caracteres", raw.chars().count());
            println!("Inicio:\n{}", raw.chars().take(300).collect::<String>());
            let analysis = parse_analysis(&raw);
            println!("\n== 4) ANÁLISIS ESTRUCTURADO ==");
            println!("Puntuación: {}", analysis.puntuacion);
            println!("Resumen: {}", analysis.resumen.chars().take(200).collect::<String>());
            println!("Categorías ({}):", analysis.categorias.len());
            for c in &analysis.categorias {
                println!("  - {:<24} {}  {}", c.name, c.nota, c.comentario.chars().take(80).collect::<String>());
            }
            println!("Aciertos: {}", analysis.aciertos.len());
            for a in &analysis.aciertos {
                println!("  + {}", a.chars().take(100).collect::<String>());
            }
            println!("Errores: {}", analysis.errores.len());
            for e in &analysis.errores {
                println!("  - {}", e.chars().take(100).collect::<String>());
            }
            println!("Consejos: {}", analysis.consejos.len());
            for c in &analysis.consejos {
                println!("  * {} → {}", c.titulo, c.drill.chars().take(100).collect::<String>());
            }
            println!(
                "Progreso: mejoras={} regresiones={}",
                analysis.progreso.mejoras.len(),
                analysis.progreso.regresiones.len()
            );
            println!("\nE2E OK ✅");
        }
        Err(e) => {
            eprintln!("FALLO en el análisis del coach: {e}");
            std::process::exit(1);
        }
    }
}