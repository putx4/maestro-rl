use serde::{Deserialize, Serialize};

pub fn default_server_url() -> String {
    "http://127.0.0.1:4096".to_string()
}
pub fn default_nick() -> String {
    "lacuca3000".to_string()
}
pub fn default_rank() -> String {
    "Campeón 1".to_string()
}
pub fn default_modes() -> String {
    "2v2 y 3v3".to_string()
}
pub fn default_language() -> String {
    "es".to_string()
}
pub fn default_memory_depth() -> usize {
    10
}
pub fn default_timeout_secs() -> u64 {
    180
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoachSettings {
    #[serde(default = "default_server_url")]
    pub server_url: String,
    #[serde(default = "default_nick")]
    pub nick: String,
    #[serde(default = "default_rank")]
    pub rank: String,
    #[serde(default = "default_modes")]
    pub modes: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_memory_depth")]
    pub memory_depth: usize,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for CoachSettings {
    fn default() -> Self {
        Self {
            server_url: default_server_url(),
            nick: default_nick(),
            rank: default_rank(),
            modes: default_modes(),
            language: default_language(),
            memory_depth: default_memory_depth(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStat {
    pub name: String,
    pub team: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub demolishes: i32,
    pub boost_pickups: i32,
    #[serde(default)]
    pub boost_big_pads: i32,
    #[serde(default)]
    pub boost_small_pads: i32,
    #[serde(default)]
    pub avg_boost: f64,
    #[serde(default)]
    pub low_boost_pct: f64,
    #[serde(default)]
    pub clears: i32,
    #[serde(default)]
    pub denials: i32,
    #[serde(default)]
    pub steals: i32,
    #[serde(default)]
    pub breakout_damage: i32,
    pub match_score: i32,
    pub own_goals: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalEvent {
    pub frame: i32,
    pub player_name: String,
    pub team: i32,
    pub own_goal: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaySummary {
    pub major_version: i32,
    pub game_type: String,
    pub map_name: Option<String>,
    pub game_mode: Option<String>,
    pub match_type: Option<String>,
    pub team_size: Option<i32>,
    pub team_scores: Vec<i32>,
    pub date: Option<String>,
    pub num_frames: Option<i32>,
    pub goals: Vec<GoalEvent>,
    pub players: Vec<PlayerStat>,
}

impl ReplaySummary {
    pub fn duration_secs(&self) -> Option<f64> {
        self.num_frames.map(|f| (f as f64) / 30.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryScore {
    pub key: String,
    pub name: String,
    pub nota: f64,
    pub comentario: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Consejo {
    pub titulo: String,
    pub explicacion: String,
    pub drill: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgresoRelativo {
    pub mejoras: Vec<String>,
    pub regresiones: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub resumen: String,
    pub puntuacion: f64,
    pub categorias: Vec<CategoryScore>,
    pub aciertos: Vec<String>,
    pub errores: Vec<String>,
    pub consejos: Vec<Consejo>,
    pub progreso: ProgresoRelativo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisEntry {
    pub id: String,
    pub created_at: String,
    pub file_path: String,
    pub file_name: String,
    pub summary: ReplaySummary,
    pub raw_properties: serde_json::Value,
    pub analysis: Option<Analysis>,
    pub raw_response: Option<String>,
}

impl AnalysisEntry {
    // A list entry only carries enough info for list views.
    pub fn to_meta(&self, nick: &str) -> EntryMeta {
        let my_idx = self
            .summary
            .players
            .iter()
            .position(|p| p.name.eq_ignore_ascii_case(nick.trim()));
        let my_stats = my_idx.map(|i| self.summary.players[i].clone());
        let won = my_stats.as_ref().map(|me| {
            let my_team = me.team;
            let their_team = 1 - my_team;
            let my_score = self
                .summary
                .team_scores
                .get(my_team as usize)
                .copied()
                .unwrap_or(0);
            let their_score = self
                .summary
                .team_scores
                .get(their_team as usize)
                .copied()
                .unwrap_or(0);
            my_score > their_score
        });
        EntryMeta {
            id: self.id.clone(),
            created_at: self.created_at.clone(),
            file_name: self.file_name.clone(),
            map_name: self.summary.map_name.clone(),
            game_mode: self.summary.game_mode.clone(),
            team_scores: self.summary.team_scores.clone(),
            my_stats,
            won,
            analysis: self.analysis.as_ref().map(|a| AnalysisSummary {
                score: a.puntuacion,
                category_count: a.categorias.len(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSummary {
    pub score: f64,
    pub category_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryMeta {
    pub id: String,
    pub created_at: String,
    pub file_name: String,
    pub map_name: Option<String>,
    pub game_mode: Option<String>,
    pub team_scores: Vec<i32>,
    pub my_stats: Option<PlayerStat>,
    pub won: Option<bool>,
    pub analysis: Option<AnalysisSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub summary: ReplaySummary,
    pub raw_properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTrend {
    pub key: String,
    pub name: String,
    pub appearances: usize,
    pub last: Option<f64>,
    pub prev_avg: Option<f64>,
    pub delta: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPoint {
    pub id: String,
    pub created_at: String,
    pub file_name: String,
    pub score: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressReport {
    pub total: usize,
    pub analyzed: usize,
    pub wins: usize,
    pub losses: usize,
    pub categories: Vec<CategoryTrend>,
    pub history: Vec<ProgressPoint>,
}