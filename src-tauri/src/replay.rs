use crate::error::{AppError, Result};
use crate::models::{GoalEvent, ParseResult, PlayerStat, ReplaySummary};
use boxcars::{ActorId, Attribute, HeaderProp, ObjectId, ParserBuilder, Replay};
use std::collections::HashMap;

/// Column extracted from a PlayerStats-like struct: an array of scalars.
#[derive(Debug, Clone)]
enum Column {
    Ints(Vec<i32>),
    Strs(Vec<String>),
}

impl Column {
    fn len(&self) -> usize {
        match self {
            Column::Ints(v) => v.len(),
            Column::Strs(v) => v.len(),
        }
    }
    fn int(&self, i: usize) -> Option<i32> {
        match self {
            Column::Ints(v) => v.get(i).copied(),
            _ => None,
        }
    }
}

/// Lower-cased canonical key for a stat field.
fn norm_key(key: &str) -> &str {
    key.trim()
}

fn find<'a>(props: &'a [(String, HeaderProp)], key: &str) -> Option<&'a HeaderProp> {
    props.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn prop_int(props: &[(String, HeaderProp)], key: &str) -> Option<i32> {
    find(props, key).and_then(HeaderProp::as_i32)
}

fn prop_string(props: &[(String, HeaderProp)], key: &str) -> Option<String> {
    find(props, key)
        .and_then(|p| match p {
            HeaderProp::Str(s) | HeaderProp::Name(s) => Some(s.clone()),
            HeaderProp::Int(v) => Some(v.to_string()),
            _ => None,
        })
}

/// If a struct contains nothing but scalar columns, return them keyed.
fn as_columns(fields: &[(String, HeaderProp)]) -> Vec<(String, Column)> {
    fields
        .iter()
        .filter_map(|(k, v)| as_column(v).map(|c| (k.clone(), c)))
        .collect()
}

/// Convert an array of singleton maps into a scalar column, if possible.
fn as_column(prop: &HeaderProp) -> Option<Column> {
    match prop {
        HeaderProp::Array(items) => {
            let mut ints = Vec::with_capacity(items.len());
            let mut strs = Vec::with_capacity(items.len());
            for item in items {
                if item.len() != 1 {
                    return None;
                }
                match &item[0].1 {
                    HeaderProp::Int(v) => ints.push(*v),
                    HeaderProp::Byte { value: Some(v), .. } => {
                        strs.push(v.clone());
                    }
                    HeaderProp::Str(s) | HeaderProp::Name(s) => strs.push(s.clone()),
                    HeaderProp::Bool(b) => {
                        ints.push(if *b { 1 } else { 0 });
                    }
                    _ => return None,
                }
            }
            if !strs.is_empty() {
                if !ints.is_empty() {
                    return None;
                }
                return Some(Column::Strs(strs));
            }
            Some(Column::Ints(ints))
        }
        _ => None,
    }
}

fn is_scalar(prop: &HeaderProp) -> bool {
    matches!(
        prop,
        HeaderProp::Int(_)
            | HeaderProp::Str(_)
            | HeaderProp::Name(_)
            | HeaderProp::Byte { .. }
            | HeaderProp::Float(_)
            | HeaderProp::Bool(_)
    )
}

/// Extract any scalar with one of the stat keys from a struct's fields.
fn collect_stat_fields<'a>(
    fields: &'a [(String, HeaderProp)],
) -> Vec<(&'a str, &'a HeaderProp)> {
    fields
        .iter()
        .filter(|(k, v)| {
            is_scalar_field(k) && is_scalar(v)
        })
        .map(|(k, v)| (k.as_str(), v))
        .collect()
}

fn is_scalar_field(key: &str) -> bool {
    matches!(
        norm_key(key).to_lowercase().as_str(),
        "score"
            | "goals"
            | "assists"
            | "saves"
            | "shots"
            | "demolishes"
            | "demonflicted"
            | "boostpickups"
            | "matchscore"
            | "owngoals"
            | "playername"
            | "name"
            | "playerteam"
            | "team"
            | "platform"
            | "bot"
            | "bbot"
            | "titleid"
            | "uniqueid"
            | "profileid"
    )
}

fn scalar_int(v: &HeaderProp) -> Option<i32> {
    match v {
        HeaderProp::Int(i) => Some(*i),
        HeaderProp::Byte { value: Some(s), .. } => {
            let s = s.to_lowercase();
            match s.as_str() {
                "onlineplatform_steam" => Some(0),
                "onlineplatform_ps4" => Some(1),
                "onlineplatform_ps5" => Some(11),
                "onlineplatform_switch" => Some(3),
                "onlineplatform_epic" => Some(9),
                _ => None,
            }
        }
        HeaderProp::Bool(b) => Some(if *b { 1 } else { 0 }),
        _ => None,
    }
}

fn scalar_str(v: &HeaderProp) -> Option<&str> {
    match v {
        HeaderProp::Str(s) | HeaderProp::Name(s) => Some(s),
        _ => None,
    }
}

/// Build a PlayerStat from a struct of scalar stat fields.
fn row_from_fields(fields: &[(String, HeaderProp)]) -> Option<PlayerStat> {
    let stat_fields = collect_stat_fields(fields);
    if stat_fields.is_empty() {
        return None;
    }
    let mut row = PlayerStat {
        team: -1,
        ..Default::default()
    };
    let mut has_name = false;
    for (key, v) in &stat_fields {
        let k = norm_key(*key);
        match k.to_lowercase().as_str() {
            "score" | "matchscore" => {
                if let Some(i) = scalar_int(v) {
                    row.match_score = i;
                }
            }
            "goals" => row.goals = scalar_int(v).unwrap_or(0),
            "assists" => row.assists = scalar_int(v).unwrap_or(0),
            "saves" => row.saves = scalar_int(v).unwrap_or(0),
            "shots" => row.shots = scalar_int(v).unwrap_or(0),
            "demolishes" | "demonflicted" => row.demolishes = scalar_int(v).unwrap_or(0),
            "boostpickups" => row.boost_pickups = scalar_int(v).unwrap_or(0),
            "owngoals" => row.own_goals = scalar_int(v).unwrap_or(0),
            "name" | "playername" => {
                if let Some(s) = scalar_str(v) {
                    row.name.push_str(s);
                    has_name = true;
                }
            }
            "team" | "playerteam" => row.team = scalar_int(v).unwrap_or(-1),
            _ => {}
        }
    }
    if !has_name && row.name.is_empty() {
        return None;
    }
    Some(row)
}

/// Collect candidate player rows: every struct node across the whole tree that
/// carries several stat fields. Returns them with an id for the parent group.
fn collect_rows(prop: &HeaderProp, out: &mut Vec<(String, PlayerStat)>, group: &str) {
    match prop {
        HeaderProp::Struct { fields, .. } => {
            if let Some(row) = row_from_fields(fields) {
                if !out.iter().any(|(_, r)| r.name == row.name && row.name.len() > 0) {
                    out.push((group.to_string(), row));
                }
            }
            for (_, child) in fields {
                collect_rows(child, out, group);
            }
        }
        HeaderProp::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                let sub = format!("{}[{}]", group, i);
                for (_, child) in item {
                    collect_rows(child, out, &sub);
                }
            }
        }
        _ => {}
    }
}

/// Column layout: PlayerStats struct holds parallel arrays.
fn players_from_columns(fields: &[(String, HeaderProp)]) -> Option<Vec<PlayerStat>> {
    let cols = as_columns(fields);
    if cols.is_empty() {
        return None;
    }
    let names = cols
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("PlayerName") || k.eq_ignore_ascii_case("Name"))
        .and_then(|(_, c)| match c {
            Column::Strs(v) => Some(v.clone()),
            _ => None,
        });
    let len = names
        .as_ref()
        .map(|n| n.len())
        .unwrap_or_else(|| cols.iter().map(|(_, c)| c.len()).max().unwrap_or(0));

    let get = |k: &str| -> Option<&Column> {
        cols.iter().find(|(ck, _)| ck.eq_ignore_ascii_case(k)).map(|(_, c)| c)
    };

    let mut players = Vec::new();
    for i in 0..len {
        let name = names
            .as_ref()
            .and_then(|n| n.get(i))
            .cloned()
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        players.push(PlayerStat {
            name,
            team: get("PlayerTeam")
                .or_else(|| get("Team"))
                .and_then(|c| c.int(i))
                .unwrap_or(-1),
            goals: get("Goals").and_then(|c| c.int(i)).unwrap_or(0),
            assists: get("Assists").and_then(|c| c.int(i)).unwrap_or(0),
            saves: get("Saves").and_then(|c| c.int(i)).unwrap_or(0),
            shots: get("Shots").and_then(|c| c.int(i)).unwrap_or(0),
            demolishes: get("Demolishes").and_then(|c| c.int(i)).unwrap_or(0),
            boost_pickups: get("BoostPickups").and_then(|c| c.int(i)).unwrap_or(0),
            match_score: get("Score")
                .or_else(|| get("MatchScore"))
                .and_then(|c| c.int(i))
                .unwrap_or(0),
            own_goals: get("OwnGoals").and_then(|c| c.int(i)).unwrap_or(0),
            ..Default::default()
        });
    }
    if players.is_empty() {
        None
    } else {
        Some(players)
    }
}

fn dedupe(players: &mut Vec<PlayerStat>) {
    let mut seen = std::collections::HashSet::new();
    players.retain(|p| {
        let k = p.name.to_lowercase();
        seen.insert(k)
    });
    players.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
}

/// Accumulator for a player (a PRI actor) found while walking the network frames.
#[derive(Debug, Clone)]
struct NetPlayer {
    name: String,
    team: i32,
    goals: i32,
    assists: i32,
    saves: i32,
    shots: i32,
    score: i32,
    demolishes: i32,
    self_demolitions: i32,
    clears: i32,
    denials: i32,
    steals: i32,
    breakout_damage: i32,
}

impl Default for NetPlayer {
    fn default() -> Self {
        Self {
            name: String::new(),
            team: -1,
            goals: 0,
            assists: 0,
            saves: 0,
            shots: 0,
            score: 0,
            demolishes: 0,
            self_demolitions: 0,
            clears: 0,
            denials: 0,
            steals: 0,
            breakout_damage: 0,
        }
    }
}

/// Boost meter per car, tracked from `ReplicatedBoost` (a byte where 255 ==
/// 100%, sampled once per frame). Pad pickups are counted via `grant_count`,
/// which increments exactly once per real pickup (a meter jump from a
/// demolition respawn keeps `grant_count` untouched). Pad size is guessed by
/// the meter level right after the grant (a big pad fills to ~100%).
#[derive(Debug, Clone, Copy, Default)]
struct CarBoost {
    last: u8,
    last_grant: u8,
    samples: u64,
    sum: f64,
    low: u64,
    big_pads: u32,
    small_pads: u32,
}

/// Resolve an attribute class name (e.g. "TAGame.PRI_TA:MatchGoals") from an object id.
///
/// boxcars decodes each attribute against a class cache; the object id on every
/// `UpdatedAttribute` is an index into `Replay::objects`.
fn attr_class<'a>(replay: &'a Replay, object_id: &ObjectId) -> Option<&'a str> {
    replay.objects.get(object_id.0 as usize).map(|s| s.as_str())
}

fn pri_stat(pris: &mut HashMap<ActorId, NetPlayer>, actor: ActorId, f: impl FnOnce(&mut NetPlayer)) {
    let p = pris.entry(actor).or_default();
    f(p);
}

/// Fill in the two known teams. Scorers' teams come from the header goal
/// table; anyone left unknown is balanced onto the less populated team.
fn assign_teams(players: &mut [PlayerStat], goals: &[GoalEvent]) {
    let mut known: HashMap<String, i32> = HashMap::new();
    for g in goals {
        if g.team == 0 || g.team == 1 {
            known.insert(g.player_name.trim().to_lowercase(), g.team);
        }
    }
    let mut counts = [0usize, 0usize];
    for p in players.iter() {
        if p.team == 0 || p.team == 1 {
            counts[p.team as usize] += 1;
        }
    }
    for p in players.iter_mut() {
        if let Some(team) = known.get(&p.name.trim().to_lowercase()) {
            p.team = *team;
            let idx = p.team as usize;
            counts[idx] += 1; // coarse: may overcount but keeps raw knowns
            continue;
        }
        if p.team == 0 || p.team == 1 {
            continue;
        }
        let team = if counts[1] < counts[0] { 1 } else { 0 };
        counts[team] += 1;
        p.team = team as i32;
    }
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Extract per-player stats straight from the decoded network data.
///
/// boxcars 0.11 keeps player stats on the PRI actors as attributes
/// (`TAGame.PRI_TA:MatchGoals/Assists/Saves/Shots/Score/Demolishes`, the player
/// name at `Engine.PlayerReplicationInfo:PlayerName`), the car→player link at
/// `Engine.Pawn:PlayerReplicationInfo`, and boost from the car's meter component
/// (`TAGame.CarComponent_Boost_TA:ReplicatedBoost`) linked via
/// `CarComponent_TA:Vehicle`.
fn extract_players_network(replay: &Replay, goals: &[GoalEvent]) -> Vec<PlayerStat> {
    let mut pris: HashMap<ActorId, NetPlayer> = HashMap::new();
    let mut car_to_pri: HashMap<ActorId, ActorId> = HashMap::new();
    let mut boost_state: HashMap<ActorId, CarBoost> = HashMap::new();
    let mut comp_to_car: HashMap<ActorId, ActorId> = HashMap::new();

    let Some(nf) = &replay.network_frames else {
        return Vec::new();
    };

    for frame in &nf.frames {
        // Sample the current boost meter once per frame for every tracked car.
        for (_car, bs) in boost_state.iter_mut() {
            let pct = bs.last as f64 / 255.0 * 100.0;
            bs.samples += 1;
            bs.sum += pct;
            if pct < 20.0 {
                bs.low += 1;
            }
        }
        for ua in &frame.updated_actors {
let Some(class) = attr_class(replay, &ua.object_id) else {
                continue;
            };
            match class {
                "Engine.PlayerReplicationInfo:PlayerName" => {
                    if let Attribute::String(s) = &ua.attribute {
                        let name = s.trim().to_string();
                        if !name.is_empty() {
                            pri_stat(&mut pris, ua.actor_id, |p| p.name = name);
                        }
                    }
                }
                "TAGame.PRI_TA:TeamPaint" => {
                    if let Attribute::TeamPaint(tp) = &ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.team = i32::from(tp.team));
                    }
                }
                "TAGame.PRI_TA:MatchGoals" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.goals = v);
                    }
                }
                "TAGame.PRI_TA:MatchAssists" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.assists = v);
                    }
                }
                "TAGame.PRI_TA:MatchSaves" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.saves = v);
                    }
                }
                "TAGame.PRI_TA:MatchShots" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.shots = v);
                    }
                }
                "TAGame.PRI_TA:MatchScore" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.score = v);
                    }
                }
                "TAGame.PRI_TA:MatchDemolishes" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.demolishes = v);
                    }
                }
                "TAGame.PRI_TA:SelfDemolitions" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.self_demolitions = v);
                    }
                }
                "Engine.Pawn:PlayerReplicationInfo" => {
                    // Car (pawn) → player (PRI) connection for boost attribution.
                    if let Attribute::ActiveActor(aa) = &ua.attribute {
                        if aa.active {
                            car_to_pri.insert(ua.actor_id, aa.actor);
                        }
                    }
                }
                c if c.ends_with(":Vehicle") => {
                    // Every car component (boost, dodge, jump, …) points to its
                    // car here; only the boost component gets a CarBoost state.
                    if let Attribute::ActiveActor(aa) = &ua.attribute {
                        if aa.active {
                            comp_to_car.insert(ua.actor_id, aa.actor);
                        }
                    }
                }
                "TAGame.CarComponent_Boost_TA:ReplicatedBoost" => {
                    // Meter byte 0..255 with 255 == 100%. `ReplicatedBoost`
                    // (not ReplicatedBoostAmount) is what RL replays actually record.
                    if let Attribute::ReplicatedBoost(rb) = &ua.attribute {
                        let bs = boost_state.entry(ua.actor_id).or_default();
                        // grant_count increments once per real pickup and resets
                        // to 0 when the component is (re)spawned (demolition).
                        // Only count genuine increases; a reset is not a pad.
                        if rb.grant_count > bs.last_grant {
                            let delta = u32::from(rb.grant_count - bs.last_grant);
                            if (rb.boost_amount as f32) / 255.0 >= 0.90 {
                                bs.big_pads += delta;
                            } else {
                                bs.small_pads += delta;
                            }
                        }
                        bs.last_grant = rb.grant_count;
                        bs.last = rb.boost_amount;
                    }
                }
                "TAGame.PRI_TA:PossessionClears" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.clears = v);
                    }
                }
                "TAGame.PRI_TA:PossessionDenials" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.denials = v);
                    }
                }
                "TAGame.PRI_TA:PossessionSteals" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.steals = v);
                    }
                }
                "TAGame.PRI_TA:MatchBreakoutDamage" => {
                    if let Attribute::Int(v) = ua.attribute {
                        pri_stat(&mut pris, ua.actor_id, |p| p.breakout_damage = v);
                    }
                }
                _ => {}
            }
        }
    }

    // Keep only the components that actually received `ReplicatedBoost` (right
    // now every component type maps its car via :Vehicle; the boost meter is
    // only in `boost_state`, so re-keying it by car drops dodge/jump/etc.).
    // RL respawns the boost component after each demolition, so a car can have
    // several components over the match: merge them into one state per car.
    let mut car_to_boost: HashMap<ActorId, CarBoost> = HashMap::new();
    for (comp, bs) in boost_state.iter() {
        if let Some(&car) = comp_to_car.get(comp) {
            let e = car_to_boost.entry(car).or_default();
            e.samples += bs.samples;
            e.sum += bs.sum;
            e.low += bs.low;
            e.big_pads += bs.big_pads;
            e.small_pads += bs.small_pads;
            e.last = e.last.max(bs.last);
        }
    }

    // A player's pawn is also replaced on demolition (old + new car actors
    // point to the same PRI), so merge boost from every car of the same PRI.
    let mut pri_boost: HashMap<ActorId, CarBoost> = HashMap::new();
    for (car, bs) in car_to_boost.iter() {
        if let Some(&pri) = car_to_pri.get(car) {
            let e = pri_boost.entry(pri).or_default();
            e.samples += bs.samples;
            e.sum += bs.sum;
            e.low += bs.low;
            e.big_pads += bs.big_pads;
            e.small_pads += bs.small_pads;
            e.last = e.last.max(bs.last);
        }
    }

    // Authoritative goal counts (header table) + own goals per name.
    let goals_by_name: HashMap<&str, i32> = {
        let mut m: HashMap<&str, i32> = HashMap::new();
        for g in goals {
            if !g.own_goal {
                *m.entry(g.player_name.trim()).or_insert(0) += 1;
            }
        }
        m
    };
    let own_by_name: HashMap<&str, i32> = {
        let mut m: HashMap<&str, i32> = HashMap::new();
        for g in goals {
            if g.own_goal {
                *m.entry(g.player_name.trim()).or_insert(0) += 1;
            }
        }
        m
    };

    let mut players: Vec<PlayerStat> = Vec::new();
    for (actor, p) in pris {
        if p.name.is_empty() {
            continue;
        }
        let bs = pri_boost.get(&actor);
        let (big_pads, small_pads, avg_boost, low_boost_pct, pickups) = match bs {
            Some(b) => (
                b.big_pads as i32,
                b.small_pads as i32,
                if b.samples > 0 { b.sum / b.samples as f64 } else { 0.0 },
                if b.samples > 0 { b.low as f64 * 100.0 / b.samples as f64 } else { 0.0 },
                i32::try_from(b.big_pads + b.small_pads).unwrap_or(i32::MAX),
            ),
            None => (0, 0, 0.0, 0.0, 0),
        };
        let name = p.name.trim().to_string();
        let goals = goals_by_name.get(name.as_str()).copied().unwrap_or(p.goals);
        let own_goals = own_by_name.get(name.as_str()).copied().unwrap_or(0);
        players.push(PlayerStat {
            goals,
            name,
            team: p.team,
            assists: p.assists,
            saves: p.saves,
            shots: p.shots,
            demolishes: p.demolishes + p.self_demolitions,
            boost_pickups: pickups,
            boost_big_pads: big_pads,
            boost_small_pads: small_pads,
            avg_boost: round1(avg_boost),
            low_boost_pct: round1(low_boost_pct),
            clears: p.clears,
            denials: p.denials,
            steals: p.steals,
            breakout_damage: p.breakout_damage,
            match_score: p.score,
            own_goals,
        });
    }

    // Names that only appear in the goal table (rare corner) still get a row.
    for g in goals {
        if !players.iter().any(|p| p.name.eq_ignore_ascii_case(g.player_name.trim())) {
            players.push(PlayerStat {
                name: g.player_name.trim().to_string(),
                team: g.team,
                goals: if g.own_goal { 0 } else { 1 },
                own_goals: if g.own_goal { 1 } else { 0 },
                ..Default::default()
            });
        }
    }

    dedupe(&mut players);
    assign_teams(&mut players, goals);
    players
}

/// Best-effort player list extraction. If nothing structured can be found,
/// names are recovered from goal events.
fn extract_players(props: &[(String, HeaderProp)], goals: &[GoalEvent]) -> Vec<PlayerStat> {
    let mut players: Vec<PlayerStat> = Vec::new();

    if let Some(player_stats) = find(props, "PlayerStats") {
        if let HeaderProp::Struct { fields, .. } = player_stats {
            // Column layout
            if let Some(cols) = players_from_columns(fields) {
                players = cols;
            }
            // Row layout: fields are per-player structs
            if players.is_empty() {
                for (_, v) in fields {
                    if let HeaderProp::Struct { fields: inner, .. } = v {
                        if let Some(row) = row_from_fields(inner) {
                            players.push(row);
                        }
                    }
                }
            }
        }
    }

    // Global row scan fallback
    if players.len() < 2 {
        let mut candidates: Vec<(String, PlayerStat)> = Vec::new();
        for (_, prop) in props {
            collect_rows(prop, &mut candidates, "");
        }
        if !candidates.is_empty() {
            let mut best: Vec<PlayerStat> = Vec::new();
            let mut best_count = 0usize;
            let mut groups: std::collections::BTreeMap<String, Vec<PlayerStat>> =
                std::collections::BTreeMap::new();
            for (g, row) in candidates {
                groups.entry(g).or_default().push(row);
            }
            for (_, rows) in groups {
                if rows.len() > best_count {
                    best_count = rows.len();
                    best = rows;
                }
            }
            if !best.is_empty() {
                players = best;
            }
        }
    }

    // Recover missing names / players from goal events
    for g in goals {
        if !players.iter().any(|p| p.name == g.player_name) {
            players.push(PlayerStat {
                name: g.player_name.clone(),
                team: g.team,
                goals: 1,
                ..Default::default()
            });
        } else if let Some(p) =
            players.iter_mut().find(|p| p.name == g.player_name && !g.own_goal)
        {
            p.goals = p.goals.max(1);
        }
    }

    dedupe(&mut players);
    players
}

fn extract_goals(props: &[(String, HeaderProp)]) -> Vec<GoalEvent> {
    let mut goals = Vec::new();
    if let Some(goals_prop) = find(props, "Goals") {
        if let HeaderProp::Array(items) = goals_prop {
            for item in items {
                let frame = prop_int(item, "frame").unwrap_or(0);
                let name = prop_string(item, "PlayerName").unwrap_or_else(|| "?".to_string());
                let team = prop_int(item, "PlayerTeam").unwrap_or(-1);
                let own = prop_int(item, "bOwnGoal").unwrap_or(0) == 1
                    || prop_string(item, "Victim")
                        .map(|s| s.eq_ignore_ascii_case("OwnGoal"))
                        .unwrap_or(false);
                goals.push(GoalEvent {
                    frame,
                    player_name: name,
                    team,
                    own_goal: own,
                });
            }
        }
    }
    goals.sort_by_key(|g| g.frame);
    goals
}

fn game_mode_from_type(game_type: &str, mode_prop: Option<String>) -> Option<String> {
    if let Some(m) = mode_prop {
        if let Ok(code) = m.parse::<i32>() {
            return Some(
                match code {
                    0 => "Soccer",
                    1 => "Hoops",
                    2 => "Snow Day",
                    3 => "Dropshot",
                    4 => "Volleyball",
                    _ => "Modo custom",
                }
                .to_string(),
            );
        }
        if !m.is_empty() {
            return Some(m);
        }
    }
    let lower = game_type.to_lowercase();
    if lower.contains("soccar") {
        Some("Soccer".into())
    } else if lower.contains("hoops") {
        Some("Hoops".into())
    } else if lower.contains("hockey") {
        Some("Snow Day".into())
    } else if lower.contains("volleyball") || lower.contains("beachball") {
        Some("Volleyball".into())
    } else if lower.contains("dropshot") {
        Some("Dropshot".into())
    } else {
        None
    }
}

fn build_summary(replay: &Replay) -> ReplaySummary {
    let props = &replay.properties;
    let goals = extract_goals(props);

    let mut team_scores = Vec::new();
    if let Some(s) = prop_int(props, "Team0Score") {
        team_scores.push(s);
    }
    if let Some(s) = prop_int(props, "Team1Score") {
        team_scores.push(s);
    }

    // Prefer real stats from the network data; fall back to the best-effort
    // header scan for old replays or when network data is missing.
    let mut players = extract_players_network(replay, &goals);
    if players.len() < 2 {
        players = extract_players(props, &goals);
    }
    assign_teams(&mut players, &goals);

    ReplaySummary {
        major_version: replay.major_version,
        game_type: replay.game_type.clone(),
        map_name: prop_string(props, "MapName"),
        game_mode: game_mode_from_type(&replay.game_type, prop_string(props, "GameMode")),
        match_type: prop_string(props, "MatchType"),
        team_size: prop_int(props, "TeamSize"),
        team_scores,
        date: prop_string(props, "Date"),
        num_frames: prop_int(props, "NumFrames"),
        goals,
        players,
    }
}

pub fn parse_replay_file(path: &str) -> Result<ParseResult> {
    let data = std::fs::read(path)
        .map_err(|e| AppError::Replay(format!("No se pudo leer el archivo: {e}")))?;
    // Network data must be parsed: that's where real player stats live in
    // boxcars 0.11 (PRI actor attributes), not in the header properties.
    let replay = ParserBuilder::new(&data)
        .on_error_check_crc()
        .parse()
        .map_err(|e| {
            AppError::Replay(format!(
                "No es un .replay válido de Rocket League ({e}). ¿Tal vez está corrupto o es de otra versión?"
            ))
        })?;
    let summary = build_summary(&replay);
    let raw_properties = serde_json::to_value(&replay.properties).map_err(AppError::Json)?;
    Ok(ParseResult {
        summary,
        raw_properties,
    })
}