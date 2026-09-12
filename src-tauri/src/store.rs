use crate::error::{AppError, Result};
use crate::models::{AnalysisEntry, CoachSettings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreData {
    pub settings: CoachSettings,
    pub entries: Vec<AnalysisEntry>,
}

impl Default for StoreData {
    fn default() -> Self {
        Self {
            settings: CoachSettings::default(),
            entries: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Store {
    pub data: std::sync::Mutex<StoreData>,
    pub path: std::path::PathBuf,
}

fn lock_err() -> AppError {
    AppError::Store("El almacén de datos está en mal estado (lock envenenado)".into())
}

impl Store {
    pub fn load(data_dir: &std::path::Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)
            .map_err(|e| AppError::Store(format!("No se pudo crear la carpeta de datos: {e}")))?;
        let path = data_dir.join("history.json");
        let data = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| AppError::Store(format!("No se pudo leer history.json: {e}")))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            StoreData::default()
        };
        Ok(Self {
            data: std::sync::Mutex::new(data),
            path,
        })
    }

    pub fn snapshot(&self) -> Result<StoreData> {
        let guard = self.data.lock().map_err(|_| lock_err())?;
        Ok(guard.clone())
    }

    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut StoreData),
    {
        let mut guard = self.data.lock().map_err(|_| lock_err())?;
        f(&mut guard);
        let content = serde_json::to_string_pretty(&*guard)?;
        std::fs::write(&self.path, content)
            .map_err(|e| AppError::Store(format!("No se pudo guardar history.json: {e}")))?;
        Ok(())
    }

    pub fn push_entry(&self, entry: AnalysisEntry) -> Result<()> {
        self.update(|d| d.entries.push(entry))
    }

    /// Dedupe key: same replay file and same match date are considered the
    /// same match, so re-analyzing replaces the stored entry in place and
    /// keeps its original id/created_at (preserving the progress timeline).
    fn dedupe_key(
        file_name: &str,
        match_date: &Option<String>,
    ) -> String {
        format!("{}|{}", file_name, match_date.as_deref().unwrap_or(""))
    }

    pub fn upsert_entry(&self, entry: AnalysisEntry) -> Result<()> {
        let key = Self::dedupe_key(&entry.file_name, &entry.summary.date);
        self.update(move |d| {
            let pos = d.entries.iter().position(|e| {
                Self::dedupe_key(&e.file_name, &e.summary.date) == key
            });
            match pos {
                Some(i) => {
                    let old_id = d.entries[i].id.clone();
                    let old_created_at = d.entries[i].created_at.clone();
                    let mut new_entry = entry;
                    new_entry.id = old_id;
                    new_entry.created_at = old_created_at;
                    d.entries[i] = new_entry;
                }
                None => d.entries.push(entry),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ReplaySummary;

    fn entry(file_name: &str, date: Option<&str>, id: &str, created: &str) -> AnalysisEntry {
        AnalysisEntry {
            id: id.to_string(),
            created_at: created.to_string(),
            file_path: format!("C:\\demos\\{file_name}"),
            file_name: file_name.to_string(),
            summary: ReplaySummary {
                date: date.map(|s| s.to_string()),
                ..Default::default()
            },
            raw_properties: serde_json::json!({}),
            analysis: None,
            raw_response: None,
        }
    }

    fn tmp_store() -> Store {
        let dir = std::env::temp_dir().join(format!("maestro_store_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Store::load(&dir).unwrap()
    }

    #[test]
    fn upsert_replaces_same_match_keeps_id_and_created_at() {
        let s = tmp_store();
        s.upsert_entry(entry("a.replay", Some("2026-01-01"), "id-1", "t-1"))
            .unwrap();
        s.upsert_entry(entry("a.replay", Some("2026-01-01"), "id-2", "t-2"))
            .unwrap();
        let d = s.snapshot().unwrap();
        assert_eq!(d.entries.len(), 1, "no debe duplicar la misma replay+fecha");
        assert_eq!(d.entries[0].id, "id-1", "conserva el id original");
        assert_eq!(d.entries[0].created_at, "t-1", "conserva createdAt original");
    }

    #[test]
    fn different_date_or_file_are_separate_entries() {
        let s = tmp_store();
        s.upsert_entry(entry("a.replay", Some("2026-01-01"), "id-1", "t-1"))
            .unwrap();
        s.upsert_entry(entry("a.replay", Some("2026-01-02"), "id-2", "t-2"))
            .unwrap();
        s.upsert_entry(entry("b.replay", Some("2026-01-01"), "id-3", "t-3"))
            .unwrap();
        let d = s.snapshot().unwrap();
        assert_eq!(d.entries.len(), 3);
    }
}