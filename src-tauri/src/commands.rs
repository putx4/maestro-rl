use crate::coach::{build_memory_digest, build_prompt, parse_analysis, CoachClient};
use crate::error::{AppError, Result};
use crate::models::*;
use crate::replay::parse_replay_file;
use crate::store::Store;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

pub struct AppState {
    pub store: Store,
}

fn emit(app: &AppHandle, stage: &str) {
    let _ = app.emit("coach-stage", stage);
}

fn file_name_of(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "replay".to_string())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<CoachSettings> {
    Ok(state.store.snapshot()?.settings)
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: CoachSettings) -> Result<()> {
    if !settings.server_url.starts_with("http://")
        && !settings.server_url.starts_with("https://")
    {
        return Err(AppError::Store(
            "La URL del servidor debe empezar con http:// o https://".into(),
        ));
    }
    state.store.update(|d| d.settings = settings)?;
    Ok(())
}

#[tauri::command]
pub fn parse_replay(path: String) -> Result<ParseResult> {
    parse_replay_file(&path)
}

#[tauri::command]
pub async fn analyze_replay(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Analysis> {
    emit(&app, "parsing");
    let parsed = parse_replay_file(&path)?;

    let snap = state.store.snapshot()?;
    let settings = snap.settings.clone();
    let digest =
        build_memory_digest(&snap.entries, &settings.nick, settings.memory_depth);

    emit(&app, "connecting");
    let client = CoachClient::new(&settings.server_url, settings.timeout_secs);
    let prompt = build_prompt(&settings, &parsed.summary, &parsed.raw_properties, &digest);

    emit(&app, "analyzing");
    let raw = client.analyze(&prompt).await?;
    let analysis = parse_analysis(&raw);

    emit(&app, "saving");
    let entry = AnalysisEntry {
        id: Uuid::new_v4().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        file_path: path.clone(),
        file_name: file_name_of(&path),
        summary: parsed.summary,
        raw_properties: parsed.raw_properties,
        analysis: Some(analysis.clone()),
        raw_response: Some(raw),
    };
    state.store.upsert_entry(entry)?;
    Ok(analysis)
}

#[tauri::command]
pub async fn reanalyze_entry(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Analysis> {
    let snap = state.store.snapshot()?;
    let base = snap
        .entries
        .iter()
        .find(|e| e.id == id)
        .cloned()
        .ok_or_else(|| AppError::Store("No se encontró esa replay en el historial".into()))?;

    let settings = snap.settings.clone();
    let digest = build_memory_digest(&snap.entries, &settings.nick, settings.memory_depth);

    emit(&app, "connecting");
    let client = CoachClient::new(&settings.server_url, settings.timeout_secs);
    let prompt = build_prompt(&settings, &base.summary, &base.raw_properties, &digest);

    emit(&app, "analyzing");
    let raw = client.analyze(&prompt).await?;
    let analysis = parse_analysis(&raw);

    emit(&app, "saving");
    state.store.update(|d| {
        if let Some(e) = d.entries.iter_mut().find(|e| e.id == id) {
            e.analysis = Some(analysis.clone());
            e.raw_response = Some(raw);
        }
    })?;
    Ok(analysis)
}

#[tauri::command]
pub fn list_entries(state: State<AppState>) -> Result<Vec<EntryMeta>> {
    let snap = state.store.snapshot()?;
    Ok(snap
        .entries
        .iter()
        .rev()
        .map(|e| e.to_meta(&snap.settings.nick))
        .collect())
}

#[tauri::command]
pub fn get_entry(id: String, state: State<AppState>) -> Result<AnalysisEntry> {
    let snap = state.store.snapshot()?;
    snap.entries
        .iter()
        .find(|e| e.id == id)
        .cloned()
        .ok_or_else(|| AppError::Store("No se encontró esa replay en el historial".into()))
}

#[tauri::command]
pub fn delete_entry(id: String, state: State<AppState>) -> Result<()> {
    state.store.update(|d| d.entries.retain(|e| e.id != id))?;
    Ok(())
}

#[tauri::command]
pub fn clear_history(state: State<AppState>) -> Result<()> {
    state.store.update(|d| d.entries.clear())?;
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
pub async fn test_connection(state: State<'_, AppState>) -> Result<TestResult> {
    let snap = state.store.snapshot()?;
    let client = CoachClient::new(&snap.settings.server_url, snap.settings.timeout_secs);
    let session = client.create_session().await?;
    client.delete_session(&session).await;
    Ok(TestResult {
        ok: true,
        message: format!(
            "Conexión OK con opencode en {} ({})",
            snap.settings.server_url, snap.settings.nick
        ),
    })
}

#[tauri::command]
pub fn get_progress(state: State<AppState>) -> Result<ProgressReport> {
    let snap = state.store.snapshot()?;
    let nick = snap.settings.nick.clone();

    let analyzed: Vec<&AnalysisEntry> = snap
        .entries
        .iter()
        .filter(|e| e.analysis.is_some())
        .collect();

    let total = snap.entries.len();
    let analyzed_n = analyzed.len();

    let wins = analyzed
        .iter()
        .filter(|e| e.to_meta(&nick).won == Some(true))
        .count();
    let losses = analyzed
        .iter()
        .filter(|e| e.to_meta(&nick).won == Some(false))
        .count();

    // name lookup per canonical key
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    // sequence per category, insertion order = chronological
    let mut series: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();
    for e in &analyzed {
        if let Some(a) = &e.analysis {
            for c in &a.categorias {
                names.entry(c.key.clone()).or_insert_with(|| c.name.clone());
                if let Some(v) = series.get_mut(&c.key) {
                    v.push((e.created_at.clone(), c.nota));
                } else {
                    series.insert(c.key.clone(), vec![(e.created_at.clone(), c.nota)]);
                }
            }
        }
    }

    let mut categories: Vec<CategoryTrend> = Vec::new();
    for (key, mut seq) in series {
        seq.sort_by(|a, b| a.0.cmp(&b.0));
        let last = seq.last().map(|(_, n)| *n);
        let prev = {
            let before: Vec<&f64> = seq.iter().rev().skip(1).map(|(_, n)| n).collect();
            if before.is_empty() {
                None
            } else {
                Some(before.iter().copied().sum::<f64>() / before.len() as f64)
            }
        };
        let delta = match (last, prev) {
            (Some(l), Some(p)) => Some(l - p),
            _ => None,
        };
        categories.push(CategoryTrend {
            key: key.clone(),
            name: names.get(&key).cloned().unwrap_or_else(|| key.clone()),
            appearances: seq.len(),
            last,
            prev_avg: prev.map(round1),
            delta: delta.map(round1),
        });
    }
    categories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let mut history: Vec<ProgressPoint> = analyzed
        .iter()
        .map(|e| ProgressPoint {
            id: e.id.clone(),
            created_at: e.created_at.clone(),
            file_name: e.file_name.clone(),
            score: e.analysis.as_ref().map(|a| a.puntuacion).unwrap_or(0.0),
        })
        .collect();
    history.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    Ok(ProgressReport {
        total,
        analyzed: analyzed_n,
        wins,
        losses,
        categories,
        history,
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayFileInfo {
    pub path: String,
    pub name: String,
    pub modified_at: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentReplays {
    pub folder: Option<String>,
    pub replays: Vec<ReplayFileInfo>,
}

/// Candidate roots for the Rocket League replays folder. Covers the default
/// and the OneDrive-relocated "Documentos" used on many Spanish setups.
fn replay_root_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from));
    let Some(home) = home else {
        return out;
    };
    for docs in [
        home.join("OneDrive").join("Documentos"),
        home.join("OneDrive").join("Documents"),
        home.join("Documents"),
        home.join("Documentos"),
    ] {
        out.push(docs.join("My Games").join("Rocket League").join("TAGame"));
    }
    out
}

fn rfc3339(t: SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
}

/// Lists the most recently modified `.replay` files found in the Rocket
/// League replays folder, sorted newest first. Never fails: if nothing is
/// found, `folder` stays None and the list is empty.
#[tauri::command]
pub fn recent_replays(limit: Option<usize>) -> Result<RecentReplays> {
    let limit = limit.unwrap_or(30).max(1);
    let mut found: Vec<(SystemTime, PathBuf)> = Vec::new();
    let mut folder: Option<String> = None;

    for tagame in replay_root_candidates() {
        if !tagame.is_dir() {
            continue;
        }
        if folder.is_none() {
            folder = Some(tagame.display().to_string());
        }
        let Ok(entries) = std::fs::read_dir(&tagame) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.to_lowercase().starts_with("demos") || !entry.path().is_dir() {
                continue;
            }
            let Ok(replays) = std::fs::read_dir(entry.path()) else {
                continue;
            };
            for r in replays.flatten() {
                if r
                    .path()
                    .extension()
                    .map(|e| e.eq_ignore_ascii_case("replay"))
                    .unwrap_or(false)
                {
                    let mtime = r
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(SystemTime::UNIX_EPOCH);
                    found.push((mtime, r.path()));
                }
            }
        }
    }

    found.sort_by(|a, b| b.0.cmp(&a.0));
    let replays = found
        .into_iter()
        .take(limit)
        .filter_map(|(mtime, p)| {
            let meta = p.metadata().ok()?;
            let name = p.file_name()?.to_string_lossy().into_owned();
            Some(ReplayFileInfo {
                path: p.display().to_string(),
                name,
                modified_at: Some(rfc3339(mtime)),
                size_bytes: meta.len(),
            })
        })
        .collect();

    Ok(RecentReplays { folder, replays })
}