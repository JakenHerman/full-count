use thiserror::Error;

use crate::game::{AtBatResult, BatterGameStats, BatterInfo, GameState, LineupSlot, PitcherInfo, Team};
use crate::persist;

// ── Errors ─────────────────────────────────────────────────────────────────

/// Errors returned by fallible [`App`] accessors.
#[derive(Debug, Error)]
pub enum AppError {
    /// Returned by [`App::game`] and [`App::game_mut`] when no game is in progress.
    #[error("no active game")]
    NoActiveGame,
}

// ── Screen state ───────────────────────────────────────────────────────────

/// Which top-level screen the application is currently displaying.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppScreen {
    Title,
    Setup,
    Scoring,
    Summary,
    LoadGame,
}

// ── Input modes (within Scoring screen) ───────────────────────────────────

/// Discriminates the at-bat result types that require a fielder-position sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FielderResultType {
    Groundout,
    DoublePlay,
    Flyout,
    Error,
    SacrificeFly,
}

/// Stage within the manual runner-advance flow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvanceStage {
    /// Waiting for the user to select which base the runner is currently on.
    SelectFrom,
    /// `from` has been selected; now waiting for the destination base.
    SelectTo { from: u8 },
}

/// The current input mode on the Scoring screen, controlling how keystrokes are interpreted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputMode {
    WaitingForResult,
    FielderInput {
        result_type: FielderResultType,
        buffer: String,
    },
    RbiInput {
        pending_result: AtBatResult,
        buffer: String,
        /// True when this RBI prompt was triggered by a pitch (forced walk).
        /// The pitch already pushed an undo snapshot, so handle_rbi_input
        /// should not push a second one.
        from_pitch: bool,
    },
    PitcherChange {
        name_buffer: String,
    },
    RunnerAdvance(AdvanceStage),
    SavePrompt {
        buffer: String,
    },
}

// ── Setup form types ───────────────────────────────────────────────────────

/// A single row in the lineup setup form, holding the player's name and batting average string.
#[derive(Clone, Debug, Default)]
pub struct PlayerSetupRow {
    pub name: String,
    /// Raw text entered by the user; parsed by [`parse_avg`] when starting the game.
    pub avg: String,
}

/// All data collected on the game-setup screen before a game begins.
#[derive(Clone, Debug)]
pub struct SetupData {
    pub away_name: String,
    pub home_name: String,
    pub away_lineup: Vec<PlayerSetupRow>,
    pub home_lineup: Vec<PlayerSetupRow>,
    pub away_starter: String,
    pub home_starter: String,
}

impl Default for SetupData {
    fn default() -> Self {
        SetupData {
            away_name: String::new(),
            home_name: String::new(),
            away_lineup: vec![PlayerSetupRow::default(); 9],
            home_lineup: vec![PlayerSetupRow::default(); 9],
            away_starter: String::new(),
            home_starter: String::new(),
        }
    }
}

/// Which text field within a lineup slot is focused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineupField {
    Name,
    Avg,
}

/// Identifies the currently focused field on the setup form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetupSection {
    AwayTeamName,
    HomeTeamName,
    AwayLineup(usize, LineupField),
    HomeLineup(usize, LineupField),
    AwayStarter,
    HomeStarter,
}

impl SetupSection {
    /// Returns the last focusable field in the away lineup (depends on `season-avg` feature).
    fn last_away_field() -> SetupSection {
        if cfg!(feature = "season-avg") {
            SetupSection::AwayLineup(8, LineupField::Avg)
        } else {
            SetupSection::AwayLineup(8, LineupField::Name)
        }
    }

    /// Returns the last focusable field in the home lineup (depends on `season-avg` feature).
    fn last_home_field() -> SetupSection {
        if cfg!(feature = "season-avg") {
            SetupSection::HomeLineup(8, LineupField::Avg)
        } else {
            SetupSection::HomeLineup(8, LineupField::Name)
        }
    }

    /// Returns the next setup field in tab order, wrapping from the last field back to the first.
    pub fn next(&self) -> SetupSection {
        match self {
            SetupSection::AwayTeamName => SetupSection::HomeTeamName,
            SetupSection::HomeTeamName => SetupSection::AwayLineup(0, LineupField::Name),
            SetupSection::AwayLineup(row, LineupField::Name) => {
                if cfg!(feature = "season-avg") {
                    SetupSection::AwayLineup(*row, LineupField::Avg)
                } else if *row < 8 {
                    SetupSection::AwayLineup(row + 1, LineupField::Name)
                } else {
                    SetupSection::HomeLineup(0, LineupField::Name)
                }
            }
            SetupSection::AwayLineup(row, LineupField::Avg) => {
                if *row < 8 {
                    SetupSection::AwayLineup(row + 1, LineupField::Name)
                } else {
                    SetupSection::HomeLineup(0, LineupField::Name)
                }
            }
            SetupSection::HomeLineup(row, LineupField::Name) => {
                if cfg!(feature = "season-avg") {
                    SetupSection::HomeLineup(*row, LineupField::Avg)
                } else if *row < 8 {
                    SetupSection::HomeLineup(row + 1, LineupField::Name)
                } else {
                    SetupSection::AwayStarter
                }
            }
            SetupSection::HomeLineup(row, LineupField::Avg) => {
                if *row < 8 {
                    SetupSection::HomeLineup(row + 1, LineupField::Name)
                } else {
                    SetupSection::AwayStarter
                }
            }
            SetupSection::AwayStarter => SetupSection::HomeStarter,
            SetupSection::HomeStarter => SetupSection::AwayTeamName,
        }
    }

    /// Returns the previous setup field in tab order, wrapping from the first field back to the last.
    pub fn prev(&self) -> SetupSection {
        match self {
            SetupSection::AwayTeamName => SetupSection::HomeStarter,
            SetupSection::HomeTeamName => SetupSection::AwayTeamName,
            SetupSection::AwayLineup(0, LineupField::Name) => SetupSection::HomeTeamName,
            SetupSection::AwayLineup(row, LineupField::Name) => {
                if cfg!(feature = "season-avg") {
                    SetupSection::AwayLineup(row - 1, LineupField::Avg)
                } else {
                    SetupSection::AwayLineup(row - 1, LineupField::Name)
                }
            }
            SetupSection::AwayLineup(row, LineupField::Avg) => {
                SetupSection::AwayLineup(*row, LineupField::Name)
            }
            SetupSection::HomeLineup(0, LineupField::Name) => Self::last_away_field(),
            SetupSection::HomeLineup(row, LineupField::Name) => {
                if cfg!(feature = "season-avg") {
                    SetupSection::HomeLineup(row - 1, LineupField::Avg)
                } else {
                    SetupSection::HomeLineup(row - 1, LineupField::Name)
                }
            }
            SetupSection::HomeLineup(row, LineupField::Avg) => {
                SetupSection::HomeLineup(*row, LineupField::Name)
            }
            SetupSection::AwayStarter => Self::last_home_field(),
            SetupSection::HomeStarter => SetupSection::AwayStarter,
        }
    }
}

// ── App ────────────────────────────────────────────────────────────────────

/// Top-level application state shared between the UI renderer and input handler.
pub struct App {
    pub screen: AppScreen,
    pub game: Option<GameState>,
    pub setup: SetupData,
    pub setup_cursor: SetupSection,
    pub input_mode: InputMode,
    pub status_message: Option<String>,
    pub play_log_scroll: usize,
    pub should_quit: bool,
    pub load_slots: Vec<persist::SaveSlot>,
    pub load_cursor: usize,
    pub undo_stack: Vec<GameState>,
    pub title_cursor: u8,
}

impl App {
    /// Creates a new application starting on the [`AppScreen::Title`] screen with no active game.
    pub fn new() -> Self {
        App {
            screen: AppScreen::Title,
            game: None,
            setup: SetupData::default(),
            setup_cursor: SetupSection::AwayTeamName,
            input_mode: InputMode::WaitingForResult,
            status_message: None,
            play_log_scroll: 0,
            should_quit: false,
            load_slots: Vec::new(),
            load_cursor: 0,
            undo_stack: Vec::new(),
            title_cursor: 0,
        }
    }

    /// Sets the status bar message displayed at the bottom of the Scoring screen.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    /// Clears the status bar message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Returns an immutable reference to the text of the currently focused setup field.
    pub fn current_setup_field_value(&self) -> &str {
        match &self.setup_cursor {
            SetupSection::AwayTeamName => &self.setup.away_name,
            SetupSection::HomeTeamName => &self.setup.home_name,
            SetupSection::AwayLineup(row, LineupField::Name) => &self.setup.away_lineup[*row].name,
            SetupSection::AwayLineup(row, LineupField::Avg) => &self.setup.away_lineup[*row].avg,
            SetupSection::HomeLineup(row, LineupField::Name) => &self.setup.home_lineup[*row].name,
            SetupSection::HomeLineup(row, LineupField::Avg) => &self.setup.home_lineup[*row].avg,
            SetupSection::AwayStarter => &self.setup.away_starter,
            SetupSection::HomeStarter => &self.setup.home_starter,
        }
    }

    /// Appends `c` to the currently focused setup field.
    ///
    /// For batting-average fields, only ASCII digits and `'.'` are accepted; other characters are silently dropped.
    pub fn type_char_in_setup(&mut self, c: char) {
        let is_avg = matches!(
            &self.setup_cursor,
            SetupSection::AwayLineup(_, LineupField::Avg) | SetupSection::HomeLineup(_, LineupField::Avg)
        );
        if is_avg {
            if c.is_ascii_digit() || c == '.' {
                self.setup_field_mut().push(c);
            }
        } else {
            self.setup_field_mut().push(c);
        }
    }

    /// Removes the last character from the currently focused setup field.
    pub fn backspace_in_setup(&mut self) {
        self.setup_field_mut().pop();
    }

    fn setup_field_mut(&mut self) -> &mut String {
        let cursor = self.setup_cursor.clone();
        match cursor {
            SetupSection::AwayTeamName => &mut self.setup.away_name,
            SetupSection::HomeTeamName => &mut self.setup.home_name,
            SetupSection::AwayLineup(row, LineupField::Name) => &mut self.setup.away_lineup[row].name,
            SetupSection::AwayLineup(row, LineupField::Avg) => &mut self.setup.away_lineup[row].avg,
            SetupSection::HomeLineup(row, LineupField::Name) => &mut self.setup.home_lineup[row].name,
            SetupSection::HomeLineup(row, LineupField::Avg) => &mut self.setup.home_lineup[row].avg,
            SetupSection::AwayStarter => &mut self.setup.away_starter,
            SetupSection::HomeStarter => &mut self.setup.home_starter,
        }
    }

    /// Validates the setup form, constructs both teams, and transitions to the Scoring screen.
    ///
    /// # Errors
    /// Returns an error string if any required field (team names, starting pitchers) is empty.
    pub fn start_game(&mut self) -> Result<(), String> {
        if self.setup.away_name.trim().is_empty() {
            return Err("Away team name required".into());
        }
        if self.setup.home_name.trim().is_empty() {
            return Err("Home team name required".into());
        }
        if self.setup.away_starter.trim().is_empty() {
            return Err("Away starting pitcher required".into());
        }
        if self.setup.home_starter.trim().is_empty() {
            return Err("Home starting pitcher required".into());
        }

        let away_lineup = build_lineup(&self.setup.away_lineup, "Away");
        let home_lineup = build_lineup(&self.setup.home_lineup, "Home");

        let away = Team::new(
            self.setup.away_name.trim().to_string(),
            away_lineup,
            PitcherInfo { name: self.setup.away_starter.trim().to_string() },
        );
        let home = Team::new(
            self.setup.home_name.trim().to_string(),
            home_lineup,
            PitcherInfo { name: self.setup.home_starter.trim().to_string() },
        );

        self.game = Some(GameState::new(away, home));
        self.screen = AppScreen::Scoring;
        Ok(())
    }

    /// Forces the game to end, assigns pitcher decisions, and transitions to the Summary screen.
    pub fn end_game(&mut self) {
        if let Some(ref mut game) = self.game {
            game.game_over = true;
            game.assign_decisions();
        }
        self.screen = AppScreen::Summary;
    }

    /// Returns a reference to the active game state.
    ///
    /// # Errors
    /// Returns [`AppError::NoActiveGame`] when no game is in progress.
    pub fn game(&self) -> Result<&GameState, AppError> {
        self.game.as_ref().ok_or(AppError::NoActiveGame)
    }

    /// Returns a mutable reference to the active game state.
    ///
    /// # Errors
    /// Returns [`AppError::NoActiveGame`] when no game is in progress.
    pub fn game_mut(&mut self) -> Result<&mut GameState, AppError> {
        self.game.as_mut().ok_or(AppError::NoActiveGame)
    }

    // ── Undo ──────────────────────────────────────────────────────────────

    /// Snapshots the current game state onto the undo stack, capped at 100 entries.
    ///
    /// The oldest snapshot is discarded when the cap is reached.
    pub fn push_undo(&mut self) {
        if let Some(game) = &self.game {
            let snapshot = game.clone();
            if self.undo_stack.len() >= 100 {
                self.undo_stack.remove(0);
            }
            self.undo_stack.push(snapshot);
        }
    }

    /// Restores the most recent snapshot and resets the input mode to `WaitingForResult`.
    ///
    /// Returns `false` if the undo stack is empty and no action was taken.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.game = Some(prev);
            self.input_mode = InputMode::WaitingForResult;
            true
        } else {
            false
        }
    }

    // ── Save / Load ────────────────────────────────────────────────────────

    /// Saves the current game to `~/.full-count/saves/` using a user-supplied name.
    ///
    /// # Errors
    /// Returns an error string if there is no active game or the write fails.
    pub fn save_game_named(&self, name: &str) -> Result<String, String> {
        match &self.game {
            Some(game) => persist::save_game_with_name(game, name),
            None => Err("No active game to save".into()),
        }
    }

    /// Populates the load menu with available saves from disk and switches to the LoadGame screen.
    pub fn open_load_menu(&mut self) {
        self.load_slots = persist::list_saves();
        self.load_cursor = 0;
        self.screen = AppScreen::LoadGame;
    }

    /// Loads the save file currently highlighted in the load menu and resumes scoring.
    ///
    /// # Errors
    /// Returns an error string if the saves list is empty or the file cannot be read or parsed.
    pub fn load_selected(&mut self) -> Result<(), String> {
        if self.load_slots.is_empty() {
            return Err("No saves available".into());
        }
        let path = self.load_slots[self.load_cursor].path.clone();
        let game = persist::load_game(&path)?;
        self.game = Some(game);
        self.input_mode = InputMode::WaitingForResult;
        self.play_log_scroll = 0;
        self.screen = AppScreen::Scoring;
        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn build_lineup(rows: &[PlayerSetupRow], prefix: &str) -> Vec<LineupSlot> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let name = if row.name.trim().is_empty() {
                format!("{} #{}", prefix, i + 1)
            } else {
                row.name.trim().to_string()
            };
            LineupSlot {
                info: BatterInfo { name, season_avg: parse_avg(&row.avg) },
                stats: BatterGameStats::default(),
            }
        })
        .collect()
}

fn parse_avg(s: &str) -> f32 {
    let s = s.trim().trim_start_matches('.');
    if s.is_empty() {
        return 0.0;
    }
    format!("0.{}", s).parse::<f32>().unwrap_or(0.0)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::helpers::{make_game, make_team};

    fn make_app_with_game() -> App {
        let mut app = App::new();
        app.game = Some(make_game());
        app.screen = AppScreen::Scoring;
        app
    }

    fn filled_setup() -> App {
        let mut app = App::new();
        app.setup.away_name = "Tigers".into();
        app.setup.home_name = "Cubs".into();
        app.setup.away_starter = "Pitcher A".into();
        app.setup.home_starter = "Pitcher B".into();
        for i in 0..9 {
            app.setup.away_lineup[i].name = format!("Away{}", i + 1);
            app.setup.home_lineup[i].name = format!("Home{}", i + 1);
        }
        app
    }

    // ── App::new ───────────────────────────────────────────────────────────

    #[test]
    fn test_app_new_starts_on_title() {
        let app = App::new();
        assert_eq!(app.screen, AppScreen::Title);
        assert!(app.game.is_none());
        assert!(!app.should_quit);
    }

    // ── Status bar ─────────────────────────────────────────────────────────

    #[test]
    fn test_set_status_and_clear() {
        let mut app = App::new();
        app.set_status("hello");
        assert_eq!(app.status_message.as_deref(), Some("hello"));
        app.clear_status();
        assert!(app.status_message.is_none());
    }

    // ── Setup field access ─────────────────────────────────────────────────

    #[test]
    fn test_current_setup_field_value_away_team_name() {
        let mut app = App::new();
        app.setup.away_name = "RedSox".into();
        app.setup_cursor = SetupSection::AwayTeamName;
        assert_eq!(app.current_setup_field_value(), "RedSox");
    }

    #[test]
    fn test_current_setup_field_value_home_lineup_name() {
        let mut app = App::new();
        app.setup.home_lineup[3].name = "Jones".into();
        app.setup_cursor = SetupSection::HomeLineup(3, LineupField::Name);
        assert_eq!(app.current_setup_field_value(), "Jones");
    }

    #[test]
    fn test_type_char_in_setup_name_field() {
        let mut app = App::new();
        app.setup_cursor = SetupSection::AwayTeamName;
        app.type_char_in_setup('R');
        app.type_char_in_setup('e');
        app.type_char_in_setup('d');
        assert_eq!(app.setup.away_name, "Red");
    }

    #[test]
    fn test_type_char_in_setup_avg_field_digits_only() {
        let mut app = App::new();
        app.setup_cursor = SetupSection::AwayLineup(0, LineupField::Avg);
        app.type_char_in_setup('3');
        app.type_char_in_setup('1');
        app.type_char_in_setup('5');
        app.type_char_in_setup('x'); // should be rejected
        assert_eq!(app.setup.away_lineup[0].avg, "315");
    }

    #[test]
    fn test_type_char_in_setup_avg_accepts_dot() {
        let mut app = App::new();
        app.setup_cursor = SetupSection::HomeLineup(0, LineupField::Avg);
        app.type_char_in_setup('.');
        app.type_char_in_setup('3');
        assert_eq!(app.setup.home_lineup[0].avg, ".3");
    }

    #[test]
    fn test_backspace_in_setup_removes_last_char() {
        let mut app = App::new();
        app.setup.away_name = "Reds".into();
        app.setup_cursor = SetupSection::AwayTeamName;
        app.backspace_in_setup();
        assert_eq!(app.setup.away_name, "Red");
    }

    #[test]
    fn test_backspace_in_setup_empty_field_noop() {
        let mut app = App::new();
        app.setup_cursor = SetupSection::AwayTeamName;
        app.backspace_in_setup(); // should not panic
        assert_eq!(app.setup.away_name, "");
    }

    // ── start_game ─────────────────────────────────────────────────────────

    #[test]
    fn test_start_game_missing_away_name_errors() {
        let mut app = filled_setup();
        app.setup.away_name = "".into();
        assert!(app.start_game().is_err());
    }

    #[test]
    fn test_start_game_missing_home_name_errors() {
        let mut app = filled_setup();
        app.setup.home_name = "   ".into();
        assert!(app.start_game().is_err());
    }

    #[test]
    fn test_start_game_missing_away_starter_errors() {
        let mut app = filled_setup();
        app.setup.away_starter = "".into();
        assert!(app.start_game().is_err());
    }

    #[test]
    fn test_start_game_missing_home_starter_errors() {
        let mut app = filled_setup();
        app.setup.home_starter = "".into();
        assert!(app.start_game().is_err());
    }

    #[test]
    fn test_start_game_success_transitions_to_scoring() {
        let mut app = filled_setup();
        assert!(app.start_game().is_ok());
        assert_eq!(app.screen, AppScreen::Scoring);
        assert!(app.game.is_some());
    }

    #[test]
    fn test_start_game_uses_default_player_names() {
        let mut app = filled_setup();
        // Leave all home lineup names empty
        for i in 0..9 { app.setup.home_lineup[i].name = "".into(); }
        app.start_game().unwrap();
        let game = app.game.unwrap();
        assert_eq!(game.home.lineup[0].info.name, "Home #1");
        assert_eq!(game.home.lineup[8].info.name, "Home #9");
    }

    // ── end_game ───────────────────────────────────────────────────────────

    #[test]
    fn test_end_game_sets_game_over_and_summary() {
        let mut app = make_app_with_game();
        app.end_game();
        assert_eq!(app.screen, AppScreen::Summary);
        assert!(app.game.as_ref().unwrap().game_over);
    }

    // ── game / game_mut accessors ──────────────────────────────────────────

    #[test]
    fn test_game_accessor_no_game_returns_error() {
        let app = App::new();
        assert!(matches!(app.game(), Err(AppError::NoActiveGame)));
    }

    #[test]
    fn test_game_mut_accessor_no_game_returns_error() {
        let mut app = App::new();
        assert!(matches!(app.game_mut(), Err(AppError::NoActiveGame)));
    }

    // ── Undo ───────────────────────────────────────────────────────────────

    #[test]
    fn test_push_undo_snapshots_game() {
        let mut app = make_app_with_game();
        app.push_undo();
        assert_eq!(app.undo_stack.len(), 1);
    }

    #[test]
    fn test_undo_restores_previous_state() {
        let mut app = make_app_with_game();
        let before_inning = app.game.as_ref().unwrap().inning;
        app.push_undo();
        // mutate
        app.game.as_mut().unwrap().inning = 5;
        assert!(app.undo());
        assert_eq!(app.game.as_ref().unwrap().inning, before_inning);
    }

    #[test]
    fn test_undo_empty_stack_returns_false() {
        let mut app = make_app_with_game();
        assert!(!app.undo());
    }

    #[test]
    fn test_undo_resets_input_mode() {
        let mut app = make_app_with_game();
        app.push_undo();
        app.input_mode = InputMode::PitcherChange { name_buffer: "X".into() };
        app.undo();
        assert_eq!(app.input_mode, InputMode::WaitingForResult);
    }

    #[test]
    fn test_undo_stack_capped_at_100() {
        let mut app = make_app_with_game();
        for _ in 0..110 { app.push_undo(); }
        assert_eq!(app.undo_stack.len(), 100);
    }

    // ── Save with no game ──────────────────────────────────────────────────

    #[test]
    fn test_save_game_named_no_game_errors() {
        let app = App::new();
        assert!(app.save_game_named("test").is_err());
    }

    // ── SetupSection navigation ────────────────────────────────────────────

    #[test]
    fn test_setup_section_next_wraps_around() {
        let last = SetupSection::HomeStarter;
        assert_eq!(last.next(), SetupSection::AwayTeamName);
    }

    #[test]
    fn test_setup_section_prev_wraps_around() {
        let first = SetupSection::AwayTeamName;
        assert_eq!(first.prev(), SetupSection::HomeStarter);
    }

    #[test]
    fn test_setup_section_next_sequential() {
        assert_eq!(SetupSection::AwayTeamName.next(), SetupSection::HomeTeamName);
        assert_eq!(
            SetupSection::HomeTeamName.next(),
            SetupSection::AwayLineup(0, LineupField::Name)
        );
        assert_eq!(SetupSection::AwayStarter.next(), SetupSection::HomeStarter);
    }

    #[test]
    fn test_setup_section_prev_sequential() {
        assert_eq!(SetupSection::HomeTeamName.prev(), SetupSection::AwayTeamName);
        assert_eq!(SetupSection::HomeStarter.prev(), SetupSection::AwayStarter);
    }

    // ── make_team / make_game sanity ───────────────────────────────────────

    #[test]
    fn test_make_team_has_nine_players() {
        let t = make_team("X");
        assert_eq!(t.lineup.len(), 9);
    }

    #[test]
    fn test_make_game_initial_state() {
        let g = make_game();
        assert_eq!(g.inning, 1);
        assert_eq!(g.away_total_runs(), 0);
        assert_eq!(g.home_total_runs(), 0);
    }
}
