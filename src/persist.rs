use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::game::GameState;

// ── On-disk format ─────────────────────────────────────────────────────────

/// The JSON structure written to disk when saving a game.
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    /// Unix timestamp (seconds) recorded at save time.
    pub saved_at_secs: u64,
    /// Whether this save was created with the `advanced-stats` feature enabled.
    #[serde(default)]
    pub advanced_stats: bool,
    pub game: GameState,
    /// Snapshots of the game state captured before each action, used for replay.
    ///
    /// Absent in saves created before the replay feature was added.
    #[serde(default)]
    pub snapshots: Vec<GameState>,
}

// ── Save slot (used by the load menu) ──────────────────────────────────────

/// A reference to a save file shown in the load menu.
#[derive(Debug, Clone)]
pub struct SaveSlot {
    pub path: PathBuf,
    /// Human-readable summary line shown in the UI (e.g. `"Away vs Home  |  Top 3  |  2-1"`).
    pub display: String,
}

// ── Directory ─────────────────────────────────────────────────────────────

/// Returns the default save directory: `~/.full-count/saves/`.
///
/// Falls back to the current working directory if `$HOME` is not set.
pub fn saves_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default())
        .join(".full-count")
        .join("saves")
}

// ── Save ──────────────────────────────────────────────────────────────────

/// Saves the game to the default saves directory under a sanitized version of `name`.
///
/// Returns the filename written on success (e.g. `"my-game.json"`).
///
/// # Errors
/// Returns an error string if the directory cannot be created or the file cannot be written.
pub fn save_game_with_name(
    game: &GameState,
    name: &str,
    snapshots: &[GameState],
) -> Result<String, String> {
    save_game_with_name_in_dir(game, name, snapshots, &saves_dir())
}

/// Saves the game to `dir` under a sanitized version of `name`.
///
/// This lower-level variant is exposed for tests that need a specific directory.
///
/// # Errors
/// Returns an error string if the directory cannot be created or the file cannot be written.
pub fn save_game_with_name_in_dir(
    game: &GameState,
    name: &str,
    snapshots: &[GameState],
    dir: &Path,
) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|e| format!("Cannot create save dir: {}", e))?;

    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let filename = format!("{}.json", sanitize_filename(name));
    let path = dir.join(&filename);
    write_save_file(game, secs, snapshots, &path)?;
    Ok(filename)
}

/// Save to an explicit directory with an auto-generated name (useful for tests).
#[cfg(test)]
pub fn save_game_to_dir(game: &GameState, dir: &Path) -> Result<String, String> {
    let name = format!(
        "{}-vs-{}",
        sanitize(&game.away.name),
        sanitize(&game.home.name)
    );
    save_game_with_name_in_dir(game, &name, &[], dir)
}

fn write_save_file(
    game: &GameState,
    saved_at_secs: u64,
    snapshots: &[GameState],
    path: &Path,
) -> Result<(), String> {
    let save = SaveFile {
        saved_at_secs,
        advanced_stats: cfg!(feature = "advanced-stats"),
        game: game.clone(),
        snapshots: snapshots.to_vec(),
    };
    let json =
        serde_json::to_string_pretty(&save).map_err(|e| format!("Serialization error: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Write error: {}", e))
}

// ── Load ──────────────────────────────────────────────────────────────────

/// Reads and deserializes a save file, returning the stored [`GameState`].
///
/// # Errors
/// Returns `"Read error: ..."` if the file cannot be read, or `"Parse error: ..."` if the JSON
/// cannot be deserialized into a [`SaveFile`].
/// Convenience wrapper that discards replay snapshots.
///
/// Used by tests and the `--load` CLI path where snapshots are loaded separately.
#[allow(dead_code)]
pub fn load_game(path: &Path) -> Result<GameState, String> {
    let (game, _snapshots) = load_game_full(path)?;
    Ok(game)
}

/// Reads and deserializes a save file, returning the stored [`GameState`] and replay snapshots.
///
/// # Errors
/// Returns an error string if the file cannot be read, parsed, or has a feature mismatch.
pub fn load_game_full(path: &Path) -> Result<(GameState, Vec<GameState>), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
    let save: SaveFile =
        serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))?;

    let current = cfg!(feature = "advanced-stats");
    if save.advanced_stats && !current {
        return Err("This save was created with advanced-stats enabled. \
             Recompile with: cargo build --features advanced-stats"
            .into());
    }
    if !save.advanced_stats && current {
        return Err("This save was created without advanced-stats. \
             Recompile without the advanced-stats feature to load it."
            .into());
    }

    Ok((save.game, save.snapshots))
}

// ── List ──────────────────────────────────────────────────────────────────

/// Lists all save files from the default directory, sorted newest-first.
pub fn list_saves() -> Vec<SaveSlot> {
    list_saves_in_dir(&saves_dir())
}

/// Lists all `.json` save files from `dir`, sorted newest-first by modification time.
///
/// Returns an empty `Vec` if the directory does not exist or cannot be read.
pub fn list_saves_in_dir(dir: &Path) -> Vec<SaveSlot> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();

    paths.sort_by_key(|p| {
        p.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });
    paths.reverse();

    paths
        .into_iter()
        .map(|path| {
            let display = slot_display(&path);
            SaveSlot { path, display }
        })
        .collect()
}

fn slot_display(path: &Path) -> String {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(save) = serde_json::from_str::<SaveFile>(&content) {
            let g = &save.game;
            let half = if g.half == crate::game::Half::Top {
                "Top"
            } else {
                "Bot"
            };
            return format!(
                "{} vs {}  |  {} {}  |  {}-{}",
                g.away.name,
                g.home.name,
                half,
                g.inning,
                g.away_total_runs(),
                g.home_total_runs(),
            );
        }
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown save")
        .to_string()
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// Short sanitizer used when building auto-generated names from team names.
#[cfg(test)]
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .take(16)
        .collect()
}

/// Sanitizer for user-supplied save names — keeps hyphens/underscores, longer limit.
fn sanitize_filename(name: &str) -> String {
    let s: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .take(64)
        .collect();
    // Collapse consecutive hyphens and strip leading/trailing ones
    let s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "save".to_string()
    } else {
        s
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{AtBatResult, GameState, Half};
    use crate::test_helpers::helpers::make_team;
    use serde_json::Value;

    fn make_game() -> GameState {
        GameState::new(make_team("Visitors"), make_team("Homers"))
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("full-count-tests-{}", std::process::id()))
            .join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_roundtrip_fresh_game() {
        let dir = test_dir("roundtrip_fresh");
        let game = make_game();

        let filename = save_game_to_dir(&game, &dir).unwrap();
        let loaded = load_game(&dir.join(&filename)).unwrap();

        assert_eq!(loaded.away.name, "Visitors");
        assert_eq!(loaded.home.name, "Homers");
        assert_eq!(loaded.inning, 1);
        assert_eq!(loaded.away_total_runs(), 0);
        assert_eq!(loaded.home_total_runs(), 0);
    }

    #[test]
    fn test_roundtrip_preserves_score() {
        let dir = test_dir("roundtrip_score");
        let mut game = make_game();

        // Away hits a 2-run homer
        game.bases.first = Some(0);
        game.apply_at_bat_result(AtBatResult::HomeRun, 2);

        let filename = save_game_to_dir(&game, &dir).unwrap();
        let loaded = load_game(&dir.join(&filename)).unwrap();

        assert_eq!(loaded.away_total_runs(), 2);
        assert_eq!(loaded.inning_scores[0].away_runs, 2);
    }

    #[test]
    fn test_roundtrip_preserves_inning_and_half() {
        let dir = test_dir("roundtrip_inning");
        let mut game = make_game();

        // End the top of the first (3 strikeouts)
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        // Now bottom of 1st
        assert_eq!(game.half, Half::Bottom);

        let filename = save_game_to_dir(&game, &dir).unwrap();
        let loaded = load_game(&dir.join(&filename)).unwrap();

        assert_eq!(loaded.inning, 1);
        assert_eq!(loaded.half, Half::Bottom);
        assert_eq!(loaded.outs, 0);
    }

    #[test]
    fn test_roundtrip_preserves_batting_order() {
        let dir = test_dir("roundtrip_order");
        let mut game = make_game();

        game.apply_at_bat_result(AtBatResult::Single, 0);
        game.apply_at_bat_result(AtBatResult::Single, 0);
        let expected_pos = game.away.batting_order_pos;

        let filename = save_game_to_dir(&game, &dir).unwrap();
        let loaded = load_game(&dir.join(&filename)).unwrap();

        assert_eq!(loaded.away.batting_order_pos, expected_pos);
    }

    #[test]
    fn test_roundtrip_preserves_play_log() {
        let dir = test_dir("roundtrip_log");
        let mut game = make_game();

        game.apply_at_bat_result(AtBatResult::Single, 0);
        assert_eq!(game.play_log.len(), 1);

        let filename = save_game_to_dir(&game, &dir).unwrap();
        let loaded = load_game(&dir.join(&filename)).unwrap();

        assert_eq!(loaded.play_log.len(), 1);
        assert_eq!(loaded.play_log[0].description, "1B");
    }

    #[test]
    fn test_roundtrip_preserves_pitcher_stats() {
        let dir = test_dir("roundtrip_pitcher");
        let mut game = make_game();

        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);

        let filename = save_game_to_dir(&game, &dir).unwrap();
        let loaded = load_game(&dir.join(&filename)).unwrap();

        assert_eq!(loaded.home.current_pitcher().stats.strikeouts, 2);
    }

    #[test]
    fn test_list_saves_empty_dir() {
        let dir = test_dir("list_empty");
        let slots = list_saves_in_dir(&dir);
        assert!(slots.is_empty());
    }

    #[test]
    fn test_list_saves_finds_json_files() {
        let dir = test_dir("list_finds");
        let game = make_game();
        save_game_to_dir(&game, &dir).unwrap();

        let slots = list_saves_in_dir(&dir);
        assert_eq!(slots.len(), 1);
    }

    #[test]
    fn test_list_saves_display_contains_team_names() {
        let dir = test_dir("list_display");
        let game = make_game();
        save_game_to_dir(&game, &dir).unwrap();

        let slots = list_saves_in_dir(&dir);
        assert!(
            slots[0].display.contains("Visitors"),
            "display: {}",
            slots[0].display
        );
        assert!(
            slots[0].display.contains("Homers"),
            "display: {}",
            slots[0].display
        );
    }

    #[test]
    fn test_list_saves_ignores_non_json() {
        let dir = test_dir("list_non_json");
        fs::write(dir.join("notes.txt"), "not a save").unwrap();
        fs::write(dir.join("data.csv"), "a,b,c").unwrap();

        let slots = list_saves_in_dir(&dir);
        assert!(slots.is_empty());
    }

    #[test]
    fn test_load_nonexistent_file_errors() {
        let result = load_game(Path::new("/nonexistent/path/save.json"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Read error"));
    }

    #[test]
    fn test_load_corrupt_json_errors() {
        let dir = test_dir("load_corrupt");
        let bad_path = dir.join("bad.json");
        fs::write(&bad_path, "{ not valid json }").unwrap();

        let result = load_game(&bad_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parse error"));
    }

    #[test]
    fn test_filename_uses_team_names() {
        let dir = test_dir("filename_teams");
        let game = make_game();
        let filename = save_game_to_dir(&game, &dir).unwrap();

        assert!(
            filename.starts_with("Visitors-vs-Homers"),
            "filename: {}",
            filename
        );
        assert!(filename.ends_with(".json"), "filename: {}", filename);
    }

    #[test]
    fn test_sanitize_replaces_spaces_and_symbols() {
        assert_eq!(sanitize("New York"), "New-York");
        assert_eq!(sanitize("Red Sox!"), "Red-Sox-");
        assert_eq!(sanitize("Cubs"), "Cubs");
    }

    #[test]
    fn test_sanitize_truncates_long_names() {
        let long = "a".repeat(32);
        assert_eq!(sanitize(&long).len(), 16);
    }

    fn astros_scoreability_fixture_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("mlb")
            .join("astros-scoreability-2026-04-09.json")
    }

    #[test]
    fn test_astros_scoreability_fixture_has_expected_metadata() {
        let path = astros_scoreability_fixture_path();
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path.display(), e));
        let doc: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("failed to parse fixture {}: {}", path.display(), e));

        assert_eq!(doc["date"].as_str(), Some("2026-04-09"));
        assert_eq!(doc["team_id"].as_u64(), Some(117));
        assert_eq!(doc["team_name"].as_str(), Some("Houston Astros"));
        assert!(doc["games"].is_array(), "games should be an array");
        let games_len = doc["games"].as_array().map_or(0, |g| g.len()) as u64;
        assert_eq!(doc["total_games"].as_u64(), Some(games_len));
    }

    #[test]
    fn test_astros_scoreability_has_no_unsupported_events() {
        let path = astros_scoreability_fixture_path();
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path.display(), e));
        let doc: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("failed to parse fixture {}: {}", path.display(), e));

        let mut details = Vec::new();
        if let Some(games) = doc["games"].as_array() {
            for game in games {
                let game_pk = game["game_pk"].as_u64().unwrap_or(0);
                if let Some(events) = game["unsupported_events"].as_array() {
                    for ev in events {
                        let inning = ev["inning"].as_u64().unwrap_or(0);
                        let half = ev["half"].as_str().unwrap_or("?");
                        let event_type = ev["event_type"].as_str().unwrap_or("unknown");
                        let description = ev["description"].as_str().unwrap_or("unknown");
                        details.push(format!(
                            "game_pk={game_pk}, inning={inning}, half={half}, event_type={event_type}, description={description}"
                        ));
                    }
                }
            }
        }

        assert!(
            details.is_empty(),
            "Found unsupported MLB events for Astros fixture; open GitHub issues for each:\n{}",
            details.join("\n")
        );
    }
}
