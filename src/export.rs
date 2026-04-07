//! HTML scorecard export via Askama templates.
//!
//! Call [`export_html`] with a completed [`GameState`] to render the full
//! paper-style scorecard to `~/.full-count/exports/<away>-vs-<home>.html`.

use std::fs;
use std::path::PathBuf;

use askama::Template;

use crate::game::{GameState, Half};

// ── Template context structs ───────────────────────────────────────────────

/// One at-bat cell in the batting scorecard grid.
pub struct PlayCell {
    /// The abbreviated result shown in the cell (e.g. `"1B"`, `"K"`, `"6-3"`).
    pub text: String,
    /// CSS class controlling the cell color (`"hit"`, `"walk"`, `"strikeout"`, …).
    pub class: String,
}

/// One batter's row in the batting scorecard.
pub struct BatterRow {
    pub name: String,
    /// One [`PlayCell`] per inning slot (empty cell when the batter didn't appear).
    pub inning_plays: Vec<PlayCell>,
    // Standard stats
    pub ab: u8,
    pub r: u8,
    pub h: u8,
    pub rbi: u8,
    pub bb: u8,
    pub k: u8,
    // Advanced stats (0 when the feature is off)
    pub doubles: u8,
    pub triples: u8,
    pub home_runs: u8,
    pub sb: u8,
    pub cs: u8,
}

/// One pitcher's row in the pitching table.
pub struct PitcherRow {
    pub name: String,
    /// `"W"`, `"L"`, `"S"`, or `""`.
    pub decision: String,
    pub ip: String,
    pub h: u8,
    pub r: u8,
    pub er: u8,
    pub bb: u8,
    pub k: u8,
    pub wp: u8,
    pub bf: u8,
    pub pc: u16,
}

/// One entry in the play-by-play log section.
pub struct PlayEntry {
    /// `"Top"` or `"Bot"`.
    pub half: String,
    pub inning: u8,
    pub batter: String,
    pub description: String,
    /// CSS class for `description` coloring.
    pub description_class: String,
    pub rbi: u8,
}

/// One column in the line-score table.
pub struct InningCell {
    pub away: String,
    pub home: String,
}

// ── Askama template ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "scorecard.html")]
struct ScorecardTemplate {
    away_name: String,
    home_name: String,
    away_color: String,
    home_color: String,
    // Inning numbers [1, 2, …, N] (at least 9)
    innings: Vec<u8>,
    inning_cells: Vec<InningCell>,
    away_r: u32,
    away_h: u32,
    away_e: u8,
    home_r: u32,
    home_h: u32,
    home_e: u8,
    away_batters: Vec<BatterRow>,
    home_batters: Vec<BatterRow>,
    away_totals: BatterRow,
    home_totals: BatterRow,
    away_lob: u8,
    home_lob: u8,
    away_pitchers: Vec<PitcherRow>,
    home_pitchers: Vec<PitcherRow>,
    plays: Vec<PlayEntry>,
    advanced_stats: bool,
    /// Pre-computed colspan for the totals/LOB rows.
    stat_colspan: usize,
}

// ── Builder helpers ────────────────────────────────────────────────────────

/// Returns the CSS class for a play description.
fn play_class(desc: &str) -> &'static str {
    if desc.is_empty() {
        return "empty";
    }
    match desc {
        "1B" | "2B" | "3B" | "HR" => "hit",
        "BB" | "HBP" => "walk",
        d if d.starts_with('K') => "strikeout",
        d if d.starts_with("SB") => "stolen-base",
        d if d.starts_with("CS") => "caught-stealing",
        d if d.starts_with('E') => "error",
        _ => "out",
    }
}

fn build_batter_rows(game: &GameState, is_home: bool, num_innings: usize) -> Vec<BatterRow> {
    let team = if is_home { &game.home } else { &game.away };
    let half = if is_home { Half::Bottom } else { Half::Top };
    let batting_rows = team.batting_rows();

    batting_rows
        .iter()
        .map(|slot| {
            // Collect all plays for this batter, grouped by inning (0-indexed)
            let mut by_inning: Vec<Vec<String>> = vec![Vec::new(); num_innings];
            for entry in &game.play_log {
                if entry.half == half && entry.batter_name == slot.info.name {
                    let idx = (entry.inning as usize).saturating_sub(1);
                    if idx < num_innings {
                        by_inning[idx].push(entry.description.clone());
                    }
                }
            }

            let inning_plays = by_inning
                .iter()
                .map(|plays| {
                    if plays.is_empty() {
                        PlayCell {
                            text: String::new(),
                            class: "empty".to_string(),
                        }
                    } else {
                        let text = plays.join(" | ");
                        let class = play_class(&plays[0]).to_string();
                        PlayCell { text, class }
                    }
                })
                .collect();

            BatterRow {
                name: slot.info.name.clone(),
                inning_plays,
                ab: slot.stats.at_bats,
                r: slot.stats.runs,
                h: slot.stats.hits,
                rbi: slot.stats.rbi,
                bb: slot.stats.walks,
                k: slot.stats.strikeouts,
                doubles: slot.stats.doubles,
                triples: slot.stats.triples,
                home_runs: slot.stats.home_runs,
                sb: slot.stats.stolen_bases,
                cs: slot.stats.caught_stealing,
            }
        })
        .collect()
}

fn build_totals(batters: &[BatterRow], num_innings: usize) -> BatterRow {
    BatterRow {
        name: "TOTALS".to_string(),
        inning_plays: (0..num_innings)
            .map(|_| PlayCell {
                text: String::new(),
                class: "empty".to_string(),
            })
            .collect(),
        ab: batters.iter().map(|b| b.ab as u32).sum::<u32>() as u8,
        r: batters.iter().map(|b| b.r as u32).sum::<u32>() as u8,
        h: batters.iter().map(|b| b.h as u32).sum::<u32>() as u8,
        rbi: batters.iter().map(|b| b.rbi as u32).sum::<u32>() as u8,
        bb: batters.iter().map(|b| b.bb as u32).sum::<u32>() as u8,
        k: batters.iter().map(|b| b.k as u32).sum::<u32>() as u8,
        doubles: batters.iter().map(|b| b.doubles as u32).sum::<u32>() as u8,
        triples: batters.iter().map(|b| b.triples as u32).sum::<u32>() as u8,
        home_runs: batters.iter().map(|b| b.home_runs as u32).sum::<u32>() as u8,
        sb: batters.iter().map(|b| b.sb as u32).sum::<u32>() as u8,
        cs: batters.iter().map(|b| b.cs as u32).sum::<u32>() as u8,
    }
}

fn build_pitcher_rows(game: &GameState, is_home: bool) -> Vec<PitcherRow> {
    let team = if is_home { &game.home } else { &game.away };
    team.pitchers
        .iter()
        .map(|p| PitcherRow {
            name: p.info.name.clone(),
            decision: p
                .decision
                .map(|d| d.label().to_string())
                .unwrap_or_default(),
            ip: p.stats.ip_display(),
            h: p.stats.hits_allowed,
            r: p.stats.runs_allowed,
            er: p.stats.earned_runs,
            bb: p.stats.walks,
            k: p.stats.strikeouts,
            wp: p.stats.wild_pitches,
            bf: p.stats.batters_faced,
            pc: p.stats.pitch_count,
        })
        .collect()
}

fn build_template(game: &GameState) -> ScorecardTemplate {
    let advanced_stats = cfg!(feature = "advanced-stats");
    let num_innings = game.inning_scores.len().max(9);
    let innings: Vec<u8> = (1..=num_innings as u8).collect();

    // Detect the "home didn't bat in final inning" case (user pressed X after top half)
    let home_x_last = game.game_over && game.half == Half::Bottom;

    let inning_cells: Vec<InningCell> = (0..num_innings)
        .map(|i| {
            if i < game.inning_scores.len() {
                let score = &game.inning_scores[i];
                let home_str = if home_x_last && i + 1 == game.inning_scores.len() {
                    "x".to_string()
                } else {
                    score.home_runs.to_string()
                };
                InningCell {
                    away: score.away_runs.to_string(),
                    home: home_str,
                }
            } else {
                InningCell {
                    away: String::new(),
                    home: String::new(),
                }
            }
        })
        .collect();

    let away_batters = build_batter_rows(game, false, num_innings);
    let home_batters = build_batter_rows(game, true, num_innings);
    let away_totals = build_totals(&away_batters, num_innings);
    let home_totals = build_totals(&home_batters, num_innings);

    let plays: Vec<PlayEntry> = game
        .play_log
        .iter()
        .map(|e| PlayEntry {
            half: match e.half {
                Half::Top => "Top".to_string(),
                Half::Bottom => "Bot".to_string(),
            },
            inning: e.inning,
            batter: e.batter_name.clone(),
            description_class: play_class(&e.description).to_string(),
            description: e.description.clone(),
            rbi: e.rbi,
        })
        .collect();

    // colspan = 1 (name) + innings + stat columns
    let stat_cols = if advanced_stats { 11 } else { 6 };
    let stat_colspan = 1 + num_innings + stat_cols;

    ScorecardTemplate {
        away_name: game.away.name.clone(),
        home_name: game.home.name.clone(),
        away_color: game.away.color.to_hex().to_string(),
        home_color: game.home.color.to_hex().to_string(),
        innings,
        inning_cells,
        away_r: game.away_total_runs() as u32,
        away_h: game.away_total_hits() as u32,
        away_e: game.errors.home, // errors charged to the fielding team
        home_r: game.home_total_runs() as u32,
        home_h: game.home_total_hits() as u32,
        home_e: game.errors.away,
        away_batters,
        home_batters,
        away_totals,
        home_totals,
        away_lob: game.away.left_on_base,
        home_lob: game.home.left_on_base,
        away_pitchers: build_pitcher_rows(game, false),
        home_pitchers: build_pitcher_rows(game, true),
        plays,
        advanced_stats,
        stat_colspan,
    }
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Returns `~/.full-count/exports/`, creating it if necessary.
pub fn exports_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default())
        .join(".full-count")
        .join("exports")
}

/// Renders the game as an HTML scorecard and writes it to the exports directory.
///
/// Returns the absolute path to the written file, or an error string.
pub fn export_html(game: &GameState) -> Result<PathBuf, String> {
    let html = build_template(game)
        .render()
        .map_err(|e| format!("template error: {e}"))?;

    let dir = exports_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("could not create exports dir: {e}"))?;

    let away = slugify(&game.away.name);
    let home = slugify(&game.home.name);
    let filename = format!("{away}-vs-{home}.html");
    let path = dir.join(&filename);

    fs::write(&path, html).map_err(|e| format!("could not write export: {e}"))?;
    Ok(path)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::helpers::make_game;

    #[test]
    fn test_render_does_not_panic_on_fresh_game() {
        let game = make_game();
        let html = build_template(&game)
            .render()
            .expect("template render failed");
        assert!(html.contains("Away"), "away team name missing");
        assert!(html.contains("Home"), "home team name missing");
        assert!(html.contains("Line Score"), "line score section missing");
        assert!(html.contains("Pitching"), "pitching section missing");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("New York Yankees"), "new-york-yankees");
        assert_eq!(slugify("Red Sox!"), "red-sox");
        assert_eq!(slugify("A's"), "a-s");
    }

    #[test]
    fn test_play_class() {
        assert_eq!(play_class("1B"), "hit");
        assert_eq!(play_class("HR"), "hit");
        assert_eq!(play_class("BB"), "walk");
        assert_eq!(play_class("K"), "strikeout");
        assert_eq!(play_class("Kl"), "strikeout");
        assert_eq!(play_class("6-3"), "out");
        assert_eq!(play_class("E6"), "error");
        assert_eq!(play_class("SB (→2B)"), "stolen-base");
        assert_eq!(play_class("CS (→2B)"), "caught-stealing");
        assert_eq!(play_class(""), "empty");
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
