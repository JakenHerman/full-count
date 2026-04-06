use thiserror::Error;

use crate::game::{AtBatResult, BatterGameStats, BatterInfo, GameState, LineupSlot, PitcherInfo, Team};
use crate::persist;

// ── Errors ─────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum AppError {
    #[error("no active game")]
    NoActiveGame,
}

// ── Screen state ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppScreen {
    Title,
    Setup,
    Scoring,
    Summary,
    LoadGame,
}

// ── Input modes (within Scoring screen) ───────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FielderResultType {
    Groundout,
    DoublePlay,
    Flyout,
    Error,
    SacrificeFly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvanceStage {
    SelectFrom,
    SelectTo { from: u8 },
}

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

#[derive(Clone, Debug, Default)]
pub struct PlayerSetupRow {
    pub name: String,
    pub avg: String,
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineupField {
    Name,
    Avg,
}

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
            SetupSection::HomeLineup(0, LineupField::Name) => {
                if cfg!(feature = "season-avg") {
                    SetupSection::AwayLineup(8, LineupField::Avg)
                } else {
                    SetupSection::AwayLineup(8, LineupField::Name)
                }
            }
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
            SetupSection::AwayStarter => {
                if cfg!(feature = "season-avg") {
                    SetupSection::HomeLineup(8, LineupField::Avg)
                } else {
                    SetupSection::HomeLineup(8, LineupField::Name)
                }
            }
            SetupSection::HomeStarter => SetupSection::AwayStarter,
        }
    }
}

// ── App ────────────────────────────────────────────────────────────────────

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

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    // Returns the string value of the currently focused setup field
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

    pub fn type_char_in_setup(&mut self, c: char) {
        let cursor = self.setup_cursor.clone();
        match cursor {
            SetupSection::AwayTeamName => self.setup.away_name.push(c),
            SetupSection::HomeTeamName => self.setup.home_name.push(c),
            SetupSection::AwayLineup(row, LineupField::Name) => {
                self.setup.away_lineup[row].name.push(c)
            }
            SetupSection::AwayLineup(row, LineupField::Avg) => {
                if c.is_ascii_digit() || c == '.' {
                    self.setup.away_lineup[row].avg.push(c);
                }
            }
            SetupSection::HomeLineup(row, LineupField::Name) => {
                self.setup.home_lineup[row].name.push(c)
            }
            SetupSection::HomeLineup(row, LineupField::Avg) => {
                if c.is_ascii_digit() || c == '.' {
                    self.setup.home_lineup[row].avg.push(c);
                }
            }
            SetupSection::AwayStarter => self.setup.away_starter.push(c),
            SetupSection::HomeStarter => self.setup.home_starter.push(c),
        }
    }

    pub fn backspace_in_setup(&mut self) {
        let cursor = self.setup_cursor.clone();
        let field: &mut String = match cursor {
            SetupSection::AwayTeamName => &mut self.setup.away_name,
            SetupSection::HomeTeamName => &mut self.setup.home_name,
            SetupSection::AwayLineup(row, LineupField::Name) => {
                &mut self.setup.away_lineup[row].name
            }
            SetupSection::AwayLineup(row, LineupField::Avg) => {
                &mut self.setup.away_lineup[row].avg
            }
            SetupSection::HomeLineup(row, LineupField::Name) => {
                &mut self.setup.home_lineup[row].name
            }
            SetupSection::HomeLineup(row, LineupField::Avg) => {
                &mut self.setup.home_lineup[row].avg
            }
            SetupSection::AwayStarter => &mut self.setup.away_starter,
            SetupSection::HomeStarter => &mut self.setup.home_starter,
        };
        field.pop();
    }

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

    pub fn end_game(&mut self) {
        if let Some(ref mut game) = self.game {
            game.game_over = true;
            game.assign_decisions();
        }
        self.screen = AppScreen::Summary;
    }

    pub fn game(&self) -> Result<&GameState, AppError> {
        self.game.as_ref().ok_or(AppError::NoActiveGame)
    }

    pub fn game_mut(&mut self) -> Result<&mut GameState, AppError> {
        self.game.as_mut().ok_or(AppError::NoActiveGame)
    }

    // ── Undo ──────────────────────────────────────────────────────────────

    /// Snapshot the current game state onto the undo stack (capped at 100 entries).
    pub fn push_undo(&mut self) {
        if let Some(game) = &self.game {
            let snapshot = game.clone();
            if self.undo_stack.len() >= 100 {
                self.undo_stack.remove(0);
            }
            self.undo_stack.push(snapshot);
        }
    }

    /// Restore the previous game state. Returns false if the stack is empty.
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

    /// Save the current game with a user-supplied name.
    pub fn save_game_named(&self, name: &str) -> Result<String, String> {
        match &self.game {
            Some(game) => persist::save_game_with_name(game, name),
            None => Err("No active game to save".into()),
        }
    }

    /// Populate the load menu with available saves and switch to the load screen.
    pub fn open_load_menu(&mut self) {
        self.load_slots = persist::list_saves();
        self.load_cursor = 0;
        self.screen = AppScreen::LoadGame;
    }

    /// Load the save currently highlighted in the load menu.
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
