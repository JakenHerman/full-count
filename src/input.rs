use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AdvanceReason, AdvanceStage, App, AppScreen, FielderResultType, InputMode, SetupSection};
use crate::game::{AtBatResult, InningOutcome, PitchEvent, PitchOutcome, PitcherInfo};

/// Routes a crossterm [`Event`] to the appropriate screen handler.
///
/// `Ctrl-C` is handled globally before dispatching to per-screen logic.
pub fn handle_event(app: &mut App, event: Event) {
    if let Event::Key(key) = event {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            app.should_quit = true;
            return;
        }
        match app.screen {
            AppScreen::Title => handle_title_key(app, key),
            AppScreen::Setup => handle_setup_key(app, key),
            AppScreen::Scoring => handle_scoring_key(app, key),
            AppScreen::Summary => handle_summary_key(app, key),
            AppScreen::LoadGame => handle_load_key(app, key),
        }
    }
}

// ── Title ────────────────────────────────────────────────────────────────────────

fn handle_title_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            if app.title_cursor > 0 {
                app.title_cursor -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            if app.title_cursor < 2 {
                app.title_cursor += 1;
            }
        }
        KeyCode::Enter => match app.title_cursor {
            0 => app.screen = AppScreen::Setup,
            1 => app.open_load_menu(),
            2 => app.should_quit = true,
            _ => {}
        },
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

// ── Setup ────────────────────────────────────────────────────────────────────────

fn handle_setup_key(app: &mut App, key: KeyEvent) {
    // F3 opens the load menu from setup
    if key.code == KeyCode::F(3) {
        app.open_load_menu();
        return;
    }
    // Color-picker fields use Left/Right to cycle instead of text input.
    if app.is_color_field() {
        match key.code {
            KeyCode::Left => app.prev_color(),
            KeyCode::Right => app.next_color(),
            KeyCode::Tab | KeyCode::Enter => {
                app.setup_cursor = app.setup_cursor.next();
                app.clear_status();
            }
            KeyCode::BackTab => {
                app.setup_cursor = app.setup_cursor.prev();
                app.clear_status();
            }
            _ => {}
        }
        return;
    }
    match key.code {
        KeyCode::Tab => {
            app.setup_cursor = app.setup_cursor.next();
            app.clear_status();
        }
        KeyCode::BackTab => {
            app.setup_cursor = app.setup_cursor.prev();
            app.clear_status();
        }
        KeyCode::Enter => {
            if app.setup_cursor == SetupSection::HomeStarter {
                match app.start_game() {
                    Ok(()) => {}
                    Err(e) => app.set_status(e),
                }
            } else {
                app.setup_cursor = app.setup_cursor.next();
            }
        }
        KeyCode::Backspace => app.backspace_in_setup(),
        KeyCode::Char(c) => app.type_char_in_setup(c),
        _ => {}
    }
}

// ── Scoring ────────────────────────────────────────────────────────────────

fn handle_scoring_key(app: &mut App, key: KeyEvent) {
    // F2 opens the save-name prompt
    if key.code == KeyCode::F(2) {
        if !matches!(app.input_mode, InputMode::SavePrompt { .. }) {
            app.input_mode = InputMode::SavePrompt { buffer: String::new() };
        }
        return;
    }
    // F3 opens the load menu from within a game
    if key.code == KeyCode::F(3) {
        app.open_load_menu();
        return;
    }
    let mode = app.input_mode.clone();
    match mode {
        InputMode::WaitingForResult => handle_waiting(app, key),
        InputMode::FielderInput { .. } => handle_fielder_input(app, key),
        InputMode::RbiInput { .. } => handle_rbi_input(app, key),
        InputMode::PitcherChange { .. } => handle_pitcher_change(app, key),
        InputMode::RunnerAdvance(_) => handle_runner_advance(app, key),
        InputMode::SavePrompt { .. } => handle_save_prompt(app, key),
    }
}

fn handle_waiting(app: &mut App, key: KeyEvent) {
    app.clear_status();
    match key.code {
        // ── Pitch events ──────────────────────────────────────────────────
        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.push_undo();
            let Ok(game) = app.game_mut() else { return };
            let outcome = game.apply_pitch(PitchEvent::Ball);
            if outcome == PitchOutcome::WalkForced {
                // Pitch snapshot already covers this; mark from_pitch so
                // handle_rbi_input won't push a second one.
                app.input_mode = InputMode::RbiInput {
                    pending_result: AtBatResult::Walk,
                    buffer: String::new(),
                    from_pitch: true,
                };
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.push_undo();
            let Ok(game) = app.game_mut() else { return };
            let outcome = game.apply_pitch(PitchEvent::Strike);
            if outcome == PitchOutcome::StrikeoutForced {
                let Ok(game) = app.game_mut() else { return };
                let io = game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
                handle_inning_outcome(app, io);
            }
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            app.push_undo();
            let Ok(game) = app.game_mut() else { return };
            game.apply_pitch(PitchEvent::Foul);
        }

        // ── Hits → ask for RBI ────────────────────────────────────────────
        KeyCode::Char('1') => rbi_prompt(app, AtBatResult::Single),
        KeyCode::Char('2') => rbi_prompt(app, AtBatResult::Double),
        KeyCode::Char('3') => rbi_prompt(app, AtBatResult::Triple),
        KeyCode::Char('h') | KeyCode::Char('H') => rbi_prompt(app, AtBatResult::HomeRun),

        // ── Strikeouts (no RBI prompt) ────────────────────────────────────
        KeyCode::Char('k') | KeyCode::Char('K') => {
            app.push_undo();
            let Ok(game) = app.game_mut() else { return };
            let io = game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
            handle_inning_outcome(app, io);
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.push_undo();
            let Ok(game) = app.game_mut() else { return };
            let io = game.apply_at_bat_result(AtBatResult::StrikeoutLooking, 0);
            handle_inning_outcome(app, io);
        }

        // ── Walk / HBP → ask for RBI (bases-loaded walk = 1 RBI) ─────────
        KeyCode::Char('w') | KeyCode::Char('W') => rbi_prompt(app, AtBatResult::Walk),
        KeyCode::Char('p') | KeyCode::Char('P') => rbi_prompt(app, AtBatResult::HitByPitch),

        // ── Fielder-sequence results ──────────────────────────────────────
        KeyCode::Char('g') | KeyCode::Char('G') => fielder_prompt(app, FielderResultType::Groundout),
        KeyCode::Char('d') | KeyCode::Char('D') => fielder_prompt(app, FielderResultType::DoublePlay),
        KeyCode::Char('o') | KeyCode::Char('O') => fielder_prompt(app, FielderResultType::Flyout),
        KeyCode::Char('e') | KeyCode::Char('E') => fielder_prompt(app, FielderResultType::Error),
        KeyCode::Char('v') | KeyCode::Char('V') => fielder_prompt(app, FielderResultType::SacrificeFly),

        // ── Fielder's choice → ask for RBI ───────────────────────────────
        KeyCode::Char('c') | KeyCode::Char('C') => rbi_prompt(app, AtBatResult::FieldersChoice),

        // ── Pitcher change ────────────────────────────────────────────────
        KeyCode::Tab => {
            app.input_mode = InputMode::PitcherChange { name_buffer: String::new() };
        }

        // ── Manual runner advance ─────────────────────────────────────────
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.input_mode = InputMode::RunnerAdvance(AdvanceStage::SelectFrom);
            app.set_status("Move runner from base: [1] [2] [3]  [Esc] cancel");
        }

        // ── Undo ──────────────────────────────────────────────────────────
        KeyCode::Char('u') | KeyCode::Char('U') => {
            if !app.undo() {
                app.set_status("Nothing to undo.");
            }
        }

        // ── Play log scroll ───────────────────────────────────────────────
        // Note: 'k' (vim up) is already bound to strikeout swinging; use ↑ to scroll up.
        KeyCode::Up => {
            if app.play_log_scroll > 0 {
                app.play_log_scroll -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let Ok(game) = app.game() else { return };
            let len = game.play_log.len();
            if len > 0 {
                app.play_log_scroll = (app.play_log_scroll + 1).min(len.saturating_sub(1));
            }
        }

        // ── End game ──────────────────────────────────────────────────────
        KeyCode::Char('x') | KeyCode::Char('X') => app.end_game(),

        // ── Quit ──────────────────────────────────────────────────────────
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,

        _ => {}
    }
}

fn handle_fielder_input(app: &mut App, key: KeyEvent) {
    if let InputMode::FielderInput { result_type, buffer } = app.input_mode.clone() {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::WaitingForResult;
            }
            KeyCode::Backspace => {
                let mut buf = buffer;
                buf.pop();
                app.input_mode = InputMode::FielderInput { result_type, buffer: buf };
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => {
                let mut buf = buffer;
                buf.push(c);
                app.input_mode = InputMode::FielderInput { result_type, buffer: buf };
            }
            KeyCode::Enter => {
                match parse_fielder_sequence(&buffer, &result_type) {
                    Ok(result) => {
                        // Double play: apply directly with 0 RBI (rare to have RBI on DP)
                        let skip_rbi = matches!(&result, AtBatResult::DoublePlay(_));
                        if skip_rbi {
                            app.push_undo();
                            let Ok(game) = app.game_mut() else { return };
                            let io = game.apply_at_bat_result(result, 0);
                            handle_inning_outcome(app, io);
                            app.input_mode = InputMode::WaitingForResult;
                        } else {
                            app.input_mode = InputMode::RbiInput {
                                pending_result: result,
                                buffer: String::new(),
                                from_pitch: false,
                            };
                        }
                    }
                    Err(e) => app.set_status(e),
                }
            }
            _ => {}
        }
    }
}

fn handle_rbi_input(app: &mut App, key: KeyEvent) {
    if let InputMode::RbiInput { pending_result, buffer, from_pitch } = app.input_mode.clone() {
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::WaitingForResult;
            }
            KeyCode::Char(c) if matches!(c, '0'..='4') => {
                app.input_mode = InputMode::RbiInput {
                    pending_result,
                    buffer: c.to_string(),
                    from_pitch,
                };
            }
            KeyCode::Backspace => {
                let mut buf = buffer;
                buf.pop();
                app.input_mode = InputMode::RbiInput { pending_result, buffer: buf, from_pitch };
            }
            KeyCode::Enter => {
                // Only push an undo snapshot if this wasn't triggered by a pitch
                // (pitch-triggered actions already pushed before apply_pitch was called).
                if !from_pitch {
                    app.push_undo();
                }
                let rbi: u8 = buffer.parse().unwrap_or(0);
                let Ok(game) = app.game_mut() else { return };
                let io = game.apply_at_bat_result(pending_result, rbi);
                handle_inning_outcome(app, io);
                app.input_mode = InputMode::WaitingForResult;
            }
            _ => {}
        }
    }
}

fn handle_pitcher_change(app: &mut App, key: KeyEvent) {
    if let InputMode::PitcherChange { name_buffer } = app.input_mode.clone() {
        if key.code == KeyCode::Enter {
            if !name_buffer.trim().is_empty() {
                let name = name_buffer.trim().to_string();
                app.push_undo();
                if let Ok(game) = app.game_mut() {
                    game.change_pitcher(PitcherInfo { name: name.clone() });
                }
                app.set_status(format!("Now pitching: {}", name));
            }
            app.input_mode = InputMode::WaitingForResult;
            return;
        }
        match update_text_buffer(name_buffer, key.code) {
            None => app.input_mode = InputMode::WaitingForResult,
            Some(buf) => app.input_mode = InputMode::PitcherChange { name_buffer: buf },
        }
    }
}

fn handle_runner_advance(app: &mut App, key: KeyEvent) {
    if let InputMode::RunnerAdvance(ref stage) = app.input_mode.clone() {
        match stage {
            AdvanceStage::SelectFrom => match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::WaitingForResult;
                    app.clear_status();
                }
                KeyCode::Char(c @ '1'..='3') => {
                    let from = c as u8 - b'0';
                    app.input_mode = InputMode::RunnerAdvance(AdvanceStage::SelectTo { from });
                    app.set_status("To base: [1] [2] [3] [H]ome  [Esc] cancel");
                }
                _ => {}
            },
            AdvanceStage::SelectTo { from } => {
                let from = *from;
                match key.code {
                    KeyCode::Esc => {
                        app.input_mode = InputMode::WaitingForResult;
                        app.clear_status();
                    }
                    KeyCode::Char(c @ '1'..='3') => {
                        let to = c as u8 - b'0';
                        app.push_undo();
                        let scored = app.game_mut().map_or(false, |g| g.advance_runner(from, to));
                        if cfg!(feature = "advanced-stats") {
                            app.input_mode = InputMode::RunnerAdvance(
                                AdvanceStage::SelectReason { from, to, scored },
                            );
                            app.set_status(
                                "Reason: [S]tolen base  [W]ild pitch  [P]assed ball  [B]alk  [O]ther",
                            );
                        } else {
                            app.input_mode = InputMode::WaitingForResult;
                            app.clear_status();
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Char('H') => {
                        app.push_undo();
                        let scored = app.game_mut().map_or(false, |g| g.advance_runner(from, 4));
                        if cfg!(feature = "advanced-stats") {
                            app.input_mode = InputMode::RunnerAdvance(
                                AdvanceStage::SelectReason { from, to: 4, scored },
                            );
                            app.set_status(
                                "Reason: [S]tolen base  [W]ild pitch  [P]assed ball  [B]alk  [O]ther",
                            );
                        } else {
                            app.input_mode = InputMode::WaitingForResult;
                            app.clear_status();
                        }
                    }
                    _ => {}
                }
            }
            AdvanceStage::SelectReason { from, to, scored } => {
                let from = *from;
                let _to = *to;
                let _scored = *scored;
                let reason = match key.code {
                    KeyCode::Esc => {
                        // Treat Esc as "Other" — the move already happened
                        Some(AdvanceReason::Other)
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => Some(AdvanceReason::StolenBase),
                    KeyCode::Char('w') | KeyCode::Char('W') => Some(AdvanceReason::WildPitch),
                    KeyCode::Char('p') | KeyCode::Char('P') => Some(AdvanceReason::PassedBall),
                    KeyCode::Char('b') | KeyCode::Char('B') => Some(AdvanceReason::Balk),
                    KeyCode::Char('o') | KeyCode::Char('O') => Some(AdvanceReason::Other),
                    _ => None,
                };
                if let Some(reason) = reason {
                    apply_advance_reason(app, &reason, from);
                    app.input_mode = InputMode::WaitingForResult;
                    app.clear_status();
                }
            }
        }
    }
}

/// Credits the appropriate stat for a categorized runner advance.
///
/// `from_base` / `to_base` are 1-indexed (4 = home). The runner has already
/// been moved by `advance_runner` before this is called.
fn apply_advance_reason(app: &mut App, reason: &AdvanceReason, from_base: u8) {
    let Ok(game) = app.game_mut() else { return };
    match reason {
        AdvanceReason::StolenBase => {
            // The runner already moved from `from_base` to the next base.
            // Look up who is now on (from_base + 1), or on all bases if
            // the destination was home (scored → no longer on a base).
            let dest = from_base + 1;
            let idx = match dest {
                2 => game.bases.second,
                3 => game.bases.third,
                // Scored (home) — can't look up on a base; fall back to
                // the most-recently-advanced batter (heuristic: previous in order).
                _ => None,
            };
            if let Some(i) = idx {
                game.batting_team_mut().lineup[i].stats.stolen_bases += 1;
            }
        }
        AdvanceReason::WildPitch => {
            game.fielding_team_mut().current_pitcher_mut().stats.wild_pitches += 1;
        }
        AdvanceReason::PassedBall | AdvanceReason::Balk | AdvanceReason::Other => {
            // Logged but no specific stat field
        }
    }
}

// ── Save prompt ────────────────────────────────────────────────────────────

fn handle_save_prompt(app: &mut App, key: KeyEvent) {
    if let InputMode::SavePrompt { buffer } = app.input_mode.clone() {
        if key.code == KeyCode::Enter {
            if !buffer.trim().is_empty() {
                match app.save_game_named(&buffer) {
                    Ok(filename) => app.set_status(format!("Saved: {}", filename)),
                    Err(e) => app.set_status(e),
                }
            }
            app.input_mode = InputMode::WaitingForResult;
            return;
        }
        match update_text_buffer(buffer, key.code) {
            None => app.input_mode = InputMode::WaitingForResult,
            Some(buf) => app.input_mode = InputMode::SavePrompt { buffer: buf },
        }
    }
}

// ── Load game ──────────────────────────────────────────────────────────────

fn handle_load_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.screen = crate::app::AppScreen::Title;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            if app.load_cursor > 0 {
                app.load_cursor -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            if !app.load_slots.is_empty() {
                app.load_cursor = (app.load_cursor + 1).min(app.load_slots.len() - 1);
            }
        }
        KeyCode::Char('g') => {
            app.load_cursor = 0;
        }
        KeyCode::Char('G') => {
            if !app.load_slots.is_empty() {
                app.load_cursor = app.load_slots.len() - 1;
            }
        }
        KeyCode::Enter => {
            match app.load_selected() {
                Ok(()) => app.set_status("Game loaded. Resume scoring."),
                Err(e) => {
                    app.screen = crate::app::AppScreen::Setup;
                    app.set_status(e);
                }
            }
        }
        _ => {}
    }
}

// ── Summary ────────────────────────────────────────────────────────────────

fn handle_summary_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn rbi_prompt(app: &mut App, result: AtBatResult) {
    app.input_mode = InputMode::RbiInput {
        pending_result: result,
        buffer: String::new(),
        from_pitch: false,
    };
}

fn fielder_prompt(app: &mut App, result_type: FielderResultType) {
    app.input_mode = InputMode::FielderInput {
        result_type,
        buffer: String::new(),
    };
}

fn handle_inning_outcome(app: &mut App, outcome: InningOutcome) {
    match outcome {
        InningOutcome::ThreeOuts => app.set_status("Side retired. [X] to end game when done."),
        InningOutcome::GameOver => app.set_status("Game over! Press [X] to view the summary."),
        InningOutcome::Continue => {}
    }
}

/// Handles the common Esc / Backspace / Char key pattern for single-line text
/// buffers. Returns `None` on Esc (caller should cancel), or `Some(updated)`
/// otherwise (Enter returns the buffer unchanged so the caller can act on it).
fn update_text_buffer(buf: String, key: KeyCode) -> Option<String> {
    match key {
        KeyCode::Esc => None,
        KeyCode::Backspace => {
            let mut b = buf;
            b.pop();
            Some(b)
        }
        KeyCode::Char(c) => {
            let mut b = buf;
            b.push(c);
            Some(b)
        }
        _ => Some(buf),
    }
}

fn parse_fielder_sequence(input: &str, result_type: &FielderResultType) -> Result<AtBatResult, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Enter fielder position(s) (e.g. 6-4-3)".into());
    }

    let positions: Result<Vec<u8>, _> = input
        .split('-')
        .map(|s| s.trim().parse::<u8>())
        .collect();

    let positions = positions.map_err(|_| "Invalid position — use digits 1–9".to_string())?;

    if positions.iter().any(|&p| p < 1 || p > 9) {
        return Err("Positions must be 1–9".into());
    }

    match result_type {
        FielderResultType::Flyout => {
            if positions.len() != 1 {
                return Err("Flyout: enter exactly one fielder (e.g. 8)".into());
            }
            Ok(AtBatResult::Flyout(positions[0]))
        }
        FielderResultType::Error => {
            if positions.len() != 1 {
                return Err("Error: enter exactly one fielder (e.g. 6)".into());
            }
            Ok(AtBatResult::Error(positions[0]))
        }
        FielderResultType::SacrificeFly => {
            if positions.len() != 1 {
                return Err("Sac fly: enter exactly one fielder (e.g. 9)".into());
            }
            Ok(AtBatResult::SacrificeFly(positions[0]))
        }
        FielderResultType::DoublePlay => {
            if positions.len() < 2 {
                return Err("DP requires 2+ positions (e.g. 6-4-3)".into());
            }
            Ok(AtBatResult::DoublePlay(positions))
        }
        FielderResultType::Groundout => {
            Ok(AtBatResult::Groundout(positions))
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::FielderResultType;
    use crate::game::AtBatResult;

    // ── parse_fielder_sequence ─────────────────────────────────────────────

    #[test]
    fn test_parse_groundout_single_fielder() {
        let r = parse_fielder_sequence("6", &FielderResultType::Groundout).unwrap();
        assert_eq!(r, AtBatResult::Groundout(vec![6]));
    }

    #[test]
    fn test_parse_groundout_multi_fielder() {
        let r = parse_fielder_sequence("6-3", &FielderResultType::Groundout).unwrap();
        assert_eq!(r, AtBatResult::Groundout(vec![6, 3]));
    }

    #[test]
    fn test_parse_groundout_three_fielders() {
        let r = parse_fielder_sequence("5-4-3", &FielderResultType::Groundout).unwrap();
        assert_eq!(r, AtBatResult::Groundout(vec![5, 4, 3]));
    }

    #[test]
    fn test_parse_flyout_single() {
        let r = parse_fielder_sequence("8", &FielderResultType::Flyout).unwrap();
        assert_eq!(r, AtBatResult::Flyout(8));
    }

    #[test]
    fn test_parse_flyout_rejects_multi() {
        assert!(parse_fielder_sequence("8-4", &FielderResultType::Flyout).is_err());
    }

    #[test]
    fn test_parse_error_single() {
        let r = parse_fielder_sequence("6", &FielderResultType::Error).unwrap();
        assert_eq!(r, AtBatResult::Error(6));
    }

    #[test]
    fn test_parse_error_rejects_multi() {
        assert!(parse_fielder_sequence("6-4", &FielderResultType::Error).is_err());
    }

    #[test]
    fn test_parse_sac_fly_single() {
        let r = parse_fielder_sequence("9", &FielderResultType::SacrificeFly).unwrap();
        assert_eq!(r, AtBatResult::SacrificeFly(9));
    }

    #[test]
    fn test_parse_sac_fly_rejects_multi() {
        assert!(parse_fielder_sequence("9-4", &FielderResultType::SacrificeFly).is_err());
    }

    #[test]
    fn test_parse_double_play_two_fielders() {
        let r = parse_fielder_sequence("6-4-3", &FielderResultType::DoublePlay).unwrap();
        assert_eq!(r, AtBatResult::DoublePlay(vec![6, 4, 3]));
    }

    #[test]
    fn test_parse_double_play_requires_two_plus() {
        assert!(parse_fielder_sequence("6", &FielderResultType::DoublePlay).is_err());
    }

    #[test]
    fn test_parse_empty_input_errors() {
        assert!(parse_fielder_sequence("", &FielderResultType::Groundout).is_err());
        assert!(parse_fielder_sequence("   ", &FielderResultType::Flyout).is_err());
    }

    #[test]
    fn test_parse_invalid_position_zero() {
        assert!(parse_fielder_sequence("0", &FielderResultType::Flyout).is_err());
    }

    #[test]
    fn test_parse_invalid_position_ten() {
        assert!(parse_fielder_sequence("10", &FielderResultType::Flyout).is_err());
    }

    #[test]
    fn test_parse_non_digit_errors() {
        assert!(parse_fielder_sequence("abc", &FielderResultType::Groundout).is_err());
    }

    #[test]
    fn test_parse_groundout_whitespace_trimmed() {
        let r = parse_fielder_sequence(" 6-3 ", &FielderResultType::Groundout).unwrap();
        assert_eq!(r, AtBatResult::Groundout(vec![6, 3]));
    }

    // ── update_text_buffer ─────────────────────────────────────────────────

    #[test]
    fn test_update_text_buffer_esc_returns_none() {
        assert!(update_text_buffer("hello".into(), KeyCode::Esc).is_none());
    }

    #[test]
    fn test_update_text_buffer_backspace_pops() {
        let r = update_text_buffer("hello".into(), KeyCode::Backspace).unwrap();
        assert_eq!(r, "hell");
    }

    #[test]
    fn test_update_text_buffer_char_appends() {
        let r = update_text_buffer("hel".into(), KeyCode::Char('p')).unwrap();
        assert_eq!(r, "help");
    }

    #[test]
    fn test_update_text_buffer_enter_returns_unchanged() {
        let r = update_text_buffer("hello".into(), KeyCode::Enter).unwrap();
        assert_eq!(r, "hello");
    }

    #[test]
    fn test_update_text_buffer_backspace_empty_noop() {
        let r = update_text_buffer(String::new(), KeyCode::Backspace).unwrap();
        assert_eq!(r, "");
    }
}
