use serde::{Deserialize, Serialize};

// ── Team color ─────────────────────────────────────────────────────────────

/// A curated palette of team colors covering all 30 MLB primary colors.
///
/// Uses true-color RGB where ANSI colors lack a good match.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamColor {
    /// Cardinals, Reds, Red Sox, Angels, Phillies, Nationals, D-backs, Braves, Twins, Guardians
    Red,
    /// Astros, Orioles, Giants, Mets
    Orange,
    /// Pirates, Brewers
    Gold,
    Yellow,
    /// Athletics
    Green,
    /// Mariners
    Teal,
    Cyan,
    /// Cubs, Dodgers, Royals, Rangers, Blue Jays
    Blue,
    /// Yankees, Rays, Tigers, Twins, Brewers, Guardians
    Navy,
    /// Rockies
    Purple,
    Magenta,
    /// Padres
    Brown,
    /// White Sox, Marlins, Pirates
    Black,
    Gray,
    #[default]
    White,
}

impl TeamColor {
    /// All available team colors, in picker order.
    pub const ALL: &'static [TeamColor] = &[
        TeamColor::Red,
        TeamColor::Orange,
        TeamColor::Gold,
        TeamColor::Yellow,
        TeamColor::Green,
        TeamColor::Teal,
        TeamColor::Cyan,
        TeamColor::Blue,
        TeamColor::Navy,
        TeamColor::Purple,
        TeamColor::Magenta,
        TeamColor::Brown,
        TeamColor::Black,
        TeamColor::Gray,
        TeamColor::White,
    ];

    /// Display name for the UI.
    pub fn name(self) -> &'static str {
        match self {
            TeamColor::Red => "Red",
            TeamColor::Orange => "Orange",
            TeamColor::Gold => "Gold",
            TeamColor::Yellow => "Yellow",
            TeamColor::Green => "Green",
            TeamColor::Teal => "Teal",
            TeamColor::Cyan => "Cyan",
            TeamColor::Blue => "Blue",
            TeamColor::Navy => "Navy",
            TeamColor::Purple => "Purple",
            TeamColor::Magenta => "Magenta",
            TeamColor::Brown => "Brown",
            TeamColor::Black => "Black",
            TeamColor::Gray => "Gray",
            TeamColor::White => "White",
        }
    }

    /// Next color in the palette, wrapping around.
    pub fn next(self) -> TeamColor {
        let idx = TeamColor::ALL.iter().position(|&c| c == self).unwrap_or(0);
        TeamColor::ALL[(idx + 1) % TeamColor::ALL.len()]
    }

    /// Previous color in the palette, wrapping around.
    pub fn prev(self) -> TeamColor {
        let idx = TeamColor::ALL.iter().position(|&c| c == self).unwrap_or(0);
        TeamColor::ALL[(idx + TeamColor::ALL.len() - 1) % TeamColor::ALL.len()]
    }

    /// Maps to a CSS hex color string for HTML export.
    pub fn to_hex(self) -> &'static str {
        match self {
            TeamColor::Red => "#dc143c",
            TeamColor::Orange => "#eb6e1f",
            TeamColor::Gold => "#fdb827",
            TeamColor::Yellow => "#f5c800",
            TeamColor::Green => "#228b22",
            TeamColor::Teal => "#008080",
            TeamColor::Cyan => "#0097a7",
            TeamColor::Blue => "#1565c0",
            TeamColor::Navy => "#003087",
            TeamColor::Purple => "#68399a",
            TeamColor::Magenta => "#c2185b",
            TeamColor::Brown => "#a56e37",
            TeamColor::Black => "#1a1a1a",
            TeamColor::Gray => "#757575",
            TeamColor::White => "#e0e0e0",
        }
    }

    /// Maps to the corresponding [`ratatui::style::Color`].
    ///
    /// Uses true-color RGB for Orange, Gold, Teal, Navy, Purple, and Brown
    /// so they look correct on modern terminals.
    pub fn to_color(self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            TeamColor::Red => Color::Red,
            TeamColor::Orange => Color::Rgb(235, 110, 31), // Astros orange
            TeamColor::Gold => Color::Rgb(253, 184, 39),   // Pirates gold
            TeamColor::Yellow => Color::Yellow,
            TeamColor::Green => Color::Green,
            TeamColor::Teal => Color::Rgb(0, 128, 128), // Mariners teal
            TeamColor::Cyan => Color::Cyan,
            TeamColor::Blue => Color::Blue,
            TeamColor::Navy => Color::Rgb(0, 48, 135), // Yankees navy
            TeamColor::Purple => Color::Rgb(104, 58, 150), // Rockies purple
            TeamColor::Magenta => Color::Magenta,
            TeamColor::Brown => Color::Rgb(165, 110, 55), // Padres brown
            TeamColor::Black => Color::DarkGray,          // visible on dark backgrounds
            TeamColor::Gray => Color::Gray,
            TeamColor::White => Color::White,
        }
    }
}

// ── Half-inning direction ──────────────────────────────────────────────────

/// Which half of an inning is currently being played.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Half {
    /// Visiting team bats.
    Top,
    /// Home team bats.
    Bottom,
}

// ── At-bat result ──────────────────────────────────────────────────────────

/// The outcome of a completed at-bat, used to update stats and advance baserunners.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtBatResult {
    Single,
    Double,
    Triple,
    HomeRun,
    StrikeoutSwinging,
    StrikeoutLooking,
    Walk,
    HitByPitch,
    /// Fielder sequence, e.g. `[6, 3]` or `[5, 4, 3]`.
    Groundout(Vec<u8>),
    /// Fielder sequence that records two outs, e.g. `[6, 4, 3]`.
    DoublePlay(Vec<u8>),
    /// The position number of the fielder who caught the ball.
    Flyout(u8),
    /// The position number of the fielder who committed the error.
    Error(u8),
    FieldersChoice,
    /// The position number of the outfielder who caught the fly.
    SacrificeFly(u8),
}

impl AtBatResult {
    /// Returns `true` for the four hit types: single, double, triple, and home run.
    pub fn is_hit(&self) -> bool {
        matches!(
            self,
            AtBatResult::Single | AtBatResult::Double | AtBatResult::Triple | AtBatResult::HomeRun
        )
    }

    /// Returns the number of outs this result records (0, 1, or 2).
    ///
    /// [`AtBatResult::DoublePlay`] is the only result that returns 2.
    pub fn records_out(&self) -> u8 {
        match self {
            AtBatResult::StrikeoutSwinging
            | AtBatResult::StrikeoutLooking
            | AtBatResult::Groundout(_)
            | AtBatResult::Flyout(_)
            | AtBatResult::FieldersChoice
            | AtBatResult::SacrificeFly(_) => 1,
            AtBatResult::DoublePlay(_) => 2,
            _ => 0,
        }
    }

    /// Returns `true` if this result counts as an official at-bat.
    ///
    /// Walks, hit-by-pitches, and sacrifice flies do **not** count.
    pub fn counts_as_at_bat(&self) -> bool {
        !matches!(
            self,
            AtBatResult::Walk | AtBatResult::HitByPitch | AtBatResult::SacrificeFly(_)
        )
    }

    /// Returns the standard scorecard abbreviation for this result (e.g. `"1B"`, `"K"`, `"6-3"`).
    pub fn display(&self) -> String {
        match self {
            AtBatResult::Single => "1B".into(),
            AtBatResult::Double => "2B".into(),
            AtBatResult::Triple => "3B".into(),
            AtBatResult::HomeRun => "HR".into(),
            AtBatResult::StrikeoutSwinging => "K".into(),
            AtBatResult::StrikeoutLooking => "Kl".into(),
            AtBatResult::Walk => "BB".into(),
            AtBatResult::HitByPitch => "HBP".into(),
            AtBatResult::Groundout(seq) => seq_display(seq),
            AtBatResult::DoublePlay(seq) => format!("{} DP", seq_display(seq)),
            AtBatResult::Flyout(p) => format!("F{}", p),
            AtBatResult::Error(p) => format!("E{}", p),
            AtBatResult::FieldersChoice => "FC".into(),
            AtBatResult::SacrificeFly(p) => format!("SF{}", p),
        }
    }
}

fn seq_display(seq: &[u8]) -> String {
    seq.iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("-")
}

// ── Game stats ─────────────────────────────────────────────────────────────

/// Cumulative hitting statistics for a single batter in one game.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BatterGameStats {
    pub at_bats: u8,
    pub runs: u8,
    pub hits: u8,
    pub rbi: u8,
    pub walks: u8,
    pub strikeouts: u8,
    pub hit_by_pitch: u8,
    #[serde(default)]
    pub doubles: u8,
    #[serde(default)]
    pub triples: u8,
    #[serde(default)]
    pub home_runs: u8,
    #[serde(default)]
    pub stolen_bases: u8,
    #[serde(default)]
    pub caught_stealing: u8,
    #[serde(default)]
    pub reached_on_error: u8,
}

/// Cumulative pitching statistics for a single pitcher appearance in one game.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PitcherGameStats {
    pub outs_recorded: u16,
    pub hits_allowed: u8,
    pub runs_allowed: u8,
    pub earned_runs: u8,
    pub walks: u8,
    pub strikeouts: u8,
    pub hit_batsmen: u8,
    pub pitch_count: u16,
    #[serde(default)]
    pub wild_pitches: u8,
    #[serde(default)]
    pub batters_faced: u8,
}

impl PitcherGameStats {
    /// Formats innings pitched in standard baseball notation (e.g. `"6.2"` for 20 outs).
    ///
    /// The digit after the decimal is the number of additional outs beyond complete innings (0, 1, or 2).
    pub fn ip_display(&self) -> String {
        format!("{}.{}", self.outs_recorded / 3, self.outs_recorded % 3)
    }
}

// ── Roster types ───────────────────────────────────────────────────────────

/// Static identifying information for a batter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatterInfo {
    pub name: String,
    /// Season batting average in the range `[0.0, 1.0]`; `0.0` means not entered.
    pub season_avg: f32,
}

impl BatterInfo {
    /// Formats the season average as a three-digit string (e.g. `".315"`).
    ///
    /// Returns `".---"` when `season_avg` is zero or negative (i.e. not entered).
    pub fn avg_display(&self) -> String {
        if self.season_avg <= 0.0 {
            ".---".to_string()
        } else {
            format!(".{:03}", (self.season_avg * 1000.0).round() as u32)
        }
    }
}

/// Static identifying information for a pitcher.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PitcherInfo {
    pub name: String,
}

/// One slot in the batting order, pairing player identity with in-game stats.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineupSlot {
    pub info: BatterInfo,
    pub stats: BatterGameStats,
}

/// A completed batter appearance that was replaced in a specific batting-order spot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatterAppearance {
    pub order_slot: usize,
    pub slot: LineupSlot,
}

/// The win/loss/save decision assigned to a pitcher at the end of the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Win,
    Loss,
    Save,
}

impl Decision {
    /// Returns the single-letter abbreviation: `"W"`, `"L"`, or `"S"`.
    pub fn label(&self) -> &'static str {
        match self {
            Decision::Win => "W",
            Decision::Loss => "L",
            Decision::Save => "S",
        }
    }
}

/// A pitcher's appearance in a game, including stats and optional decision.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PitcherAppearance {
    pub info: PitcherInfo,
    pub stats: PitcherGameStats,
    pub entered_inning: u8,
    pub decision: Option<Decision>,
}

// ── Bases ──────────────────────────────────────────────────────────────────

/// The current baserunner state, storing the batting-order index of any occupied base.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bases {
    /// Lineup index of the runner on first, or `None` if empty.
    pub first: Option<usize>,
    /// Lineup index of the runner on second, or `None` if empty.
    pub second: Option<usize>,
    /// Lineup index of the runner on third, or `None` if empty.
    pub third: Option<usize>,
}

impl Bases {
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.first.is_none() && self.second.is_none() && self.third.is_none()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Advances all runners and the batter by `bases` bases (1=single, 2=double, 3=triple, 4=HR).
    ///
    /// Returns `(runs_scored, lineup_indices_who_scored)`. Runners that reach or pass home
    /// are removed from the bases and included in the returned scorer list.
    pub fn advance_all(&mut self, bases: u8, batter_idx: usize) -> (u8, Vec<usize>) {
        let mut runs = 0u8;
        let mut scorers = Vec::new();
        let n = bases as i32;

        // Snapshot current runners (first=idx0, second=idx1, third=idx2)
        let orig = [self.first.take(), self.second.take(), self.third.take()];

        for (base_zero, runner_opt) in orig.iter().enumerate() {
            if let Some(&idx) = runner_opt.as_ref() {
                let new_pos = (base_zero as i32 + 1) + n; // 1-indexed new base
                if new_pos >= 4 {
                    runs += 1;
                    scorers.push(idx);
                } else {
                    match new_pos {
                        1 => self.first = Some(idx),
                        2 => self.second = Some(idx),
                        3 => self.third = Some(idx),
                        _ => {}
                    }
                }
            }
        }

        // Place batter
        if n >= 4 {
            runs += 1;
            scorers.push(batter_idx);
        } else if n > 0 {
            match n {
                1 => self.first = Some(batter_idx),
                2 => self.second = Some(batter_idx),
                3 => self.third = Some(batter_idx),
                _ => {}
            }
        }

        (runs, scorers)
    }

    /// Force-advances runners for a walk or hit-by-pitch — only moves runners who are forced.
    ///
    /// The batter always takes first base. A runner already on first is pushed to second,
    /// and so on. If the bases are loaded, the runner on third scores.
    /// Returns `(runs_scored, lineup_indices_who_scored)`.
    pub fn force_advance(&mut self, batter_idx: usize) -> (u8, Vec<usize>) {
        let mut runs = 0u8;
        let mut scorers = Vec::new();

        if self.first.is_some() {
            if self.second.is_some() {
                if let Some(idx) = self.third.take() {
                    runs += 1;
                    scorers.push(idx);
                }
                self.third = self.second.take();
            }
            self.second = self.first.take();
        }
        self.first = Some(batter_idx);

        (runs, scorers)
    }

    /// Resolves a fielder's choice: the lead forced runner is retired (or, if
    /// no runner is forced, the lead runner on base), any trailing forced
    /// runners advance, and the batter takes first.
    ///
    /// Returns `true` if a runner was retired on the play.
    ///
    /// # Scenarios
    /// - Runner on 1st only → runner retired at 2nd, batter to 1st.
    /// - Runners on 1st & 2nd → runner from 2nd retired at 3rd; runner from 1st
    ///   advances to 2nd; batter to 1st.
    /// - Runners on 1st & 3rd → runner from 1st retired at 2nd; runner on 3rd
    ///   unchanged; batter to 1st.
    /// - Bases loaded → runner from 3rd retired at home; runner from 2nd to 3rd;
    ///   runner from 1st to 2nd; batter to 1st. No run scores.
    /// - Runner on 2nd or 3rd only, no force → that lead runner is retired;
    ///   batter to 1st.
    /// - Empty bases → no runner is retired and the batter is NOT placed on
    ///   base. The single out recorded by [`AtBatResult::FieldersChoice`] is
    ///   attributed to the batter, leaving bases, outs, and runs internally
    ///   consistent.
    pub fn resolve_fielders_choice(&mut self, batter_idx: usize) -> bool {
        let retired;
        if self.first.is_some() {
            // A forced runner exists — retire the lead forced runner and
            // advance trailing forced runners.
            if self.second.is_some() {
                if self.third.is_some() {
                    // Bases loaded: runner from 3rd is the lead forced out.
                    self.third = self.second.take();
                } else {
                    // 1st & 2nd: runner from 2nd is the lead forced out.
                    self.second = None;
                }
                self.second = self.first.take();
            } else {
                // 1st only or 1st & 3rd: runner from 1st is the forced out.
                self.first = None;
            }
            retired = true;
        } else if self.third.is_some() {
            self.third = None;
            retired = true;
        } else if self.second.is_some() {
            self.second = None;
            retired = true;
        } else {
            retired = false;
        }
        if retired {
            self.first = Some(batter_idx);
        }
        retired
    }

    /// Resolves a standard ground-ball double play: retires the runner forced
    /// from first (in addition to the batter, which is tracked separately via
    /// outs). If first is empty, the lead runner on base is retired instead
    /// (line-drive double-off). Other runners stay in place and the batter is
    /// not placed on any base.
    ///
    /// Returns `true` if a runner was retired from the bases.
    pub fn resolve_double_play(&mut self) -> bool {
        if self.first.take().is_some() || self.third.take().is_some() {
            true
        } else {
            self.second.take().is_some()
        }
    }

    /// Moves a runner from one base to another manually.
    ///
    /// `from` and `to` are 1-indexed base numbers; use `4` for home plate (scores).
    /// Returns `true` if the runner scored. Returns `false` if there is no runner
    /// on `from`, or if `from`/`to` are out of range.
    pub fn move_runner(&mut self, from: u8, to: u8) -> bool {
        let runner = match from {
            1 => self.first.take(),
            2 => self.second.take(),
            3 => self.third.take(),
            _ => return false,
        };
        if let Some(idx) = runner {
            match to {
                1 => {
                    self.first = Some(idx);
                    false
                }
                2 => {
                    self.second = Some(idx);
                    false
                }
                3 => {
                    self.third = Some(idx);
                    false
                }
                4 => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

// ── Count ──────────────────────────────────────────────────────────────────

/// The current ball-strike count for the batter at the plate.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Count {
    pub balls: u8,
    pub strikes: u8,
}

impl Count {
    /// Increments the ball count. Returns `true` if the fourth ball causes a walk.
    pub fn add_ball(&mut self) -> bool {
        self.balls += 1;
        self.balls >= 4
    }

    /// Increments the strike count. Returns `true` if the third strike causes a strikeout.
    pub fn add_strike(&mut self) -> bool {
        self.strikes += 1;
        self.strikes >= 3
    }

    /// Records a foul ball, incrementing strikes only when below two.
    ///
    /// A foul with two strikes leaves the count unchanged (no strikeout on a foul).
    pub fn add_foul(&mut self) {
        if self.strikes < 2 {
            self.strikes += 1;
        }
    }

    /// Resets the count to 0-0.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

// ── Team ───────────────────────────────────────────────────────────────────

/// A team's roster, pitching staff, and batting-order position for one game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    #[serde(default)]
    pub color: TeamColor,
    pub lineup: Vec<LineupSlot>, // always 9 entries
    #[serde(default)]
    pub substitutions: Vec<BatterAppearance>,
    pub pitchers: Vec<PitcherAppearance>,
    pub current_pitcher_idx: usize,
    pub batting_order_pos: usize, // 0–8, wraps mod 9
    #[serde(default)]
    pub left_on_base: u8,
}

impl Team {
    /// Creates a team with a 9-player lineup, a color, and a single starting pitcher.
    pub fn new(
        name: String,
        color: TeamColor,
        lineup: Vec<LineupSlot>,
        starter: PitcherInfo,
    ) -> Self {
        Team {
            name,
            color,
            lineup,
            substitutions: Vec::new(),
            pitchers: vec![PitcherAppearance {
                info: starter,
                stats: PitcherGameStats::default(),
                entered_inning: 1,
                decision: None,
            }],
            current_pitcher_idx: 0,
            batting_order_pos: 0,
            left_on_base: 0,
        }
    }

    pub fn current_pitcher(&self) -> &PitcherAppearance {
        &self.pitchers[self.current_pitcher_idx]
    }

    pub fn current_pitcher_mut(&mut self) -> &mut PitcherAppearance {
        &mut self.pitchers[self.current_pitcher_idx]
    }

    pub fn current_batter(&self) -> &LineupSlot {
        &self.lineup[self.batting_order_pos]
    }

    pub fn current_batter_mut(&mut self) -> &mut LineupSlot {
        &mut self.lineup[self.batting_order_pos]
    }

    /// Returns batting rows in batting-order order, including any replaced hitters before the
    /// current occupant of the same lineup spot.
    pub fn batting_rows(&self) -> Vec<&LineupSlot> {
        let mut rows = Vec::with_capacity(self.lineup.len() + self.substitutions.len());
        for order_slot in 0..self.lineup.len() {
            for appearance in &self.substitutions {
                if appearance.order_slot == order_slot {
                    rows.push(&appearance.slot);
                }
            }
            rows.push(&self.lineup[order_slot]);
        }
        rows
    }

    /// Moves to the next batter, wrapping from position 8 back to 0.
    pub fn advance_batter(&mut self) {
        self.batting_order_pos = (self.batting_order_pos + 1) % self.lineup.len();
    }

    /// Replaces the current hitter with a new batter in the same batting-order spot.
    pub fn replace_current_batter(&mut self, new_batter: BatterInfo) {
        let order_slot = self.batting_order_pos;
        let replaced = self.lineup[order_slot].clone();
        self.substitutions.push(BatterAppearance {
            order_slot,
            slot: replaced,
        });
        self.lineup[order_slot] = LineupSlot {
            info: new_batter,
            stats: BatterGameStats::default(),
        };
    }

    /// Returns the total number of hits recorded across all lineup slots.
    pub fn total_hits(&self) -> u8 {
        self.lineup.iter().map(|s| s.stats.hits).sum::<u8>()
            + self
                .substitutions
                .iter()
                .map(|appearance| appearance.slot.stats.hits)
                .sum::<u8>()
    }
}

// ── Inning score ───────────────────────────────────────────────────────────

/// Runs scored by each team in a single inning.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InningScore {
    pub away_runs: u8,
    pub home_runs: u8,
}

// ── Play log ───────────────────────────────────────────────────────────────

/// A single entry in the play-by-play log.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayLogEntry {
    pub inning: u8,
    pub half: Half,
    pub batter_name: String,
    pub description: String,
    pub rbi: u8,
}

impl PlayLogEntry {
    /// Formats the entry for display, e.g. `"T3: Smith — 1B (1 RBI)"`.
    pub fn display(&self) -> String {
        let h = match self.half {
            Half::Top => "T",
            Half::Bottom => "B",
        };
        format!(
            "{}{}: {} \u{2014} {} ({} RBI)",
            h, self.inning, self.batter_name, self.description, self.rbi
        )
    }
}

// ── Pitch events and outcomes ──────────────────────────────────────────────

/// A single pitch classification used to update the count.
#[derive(Clone, Debug)]
pub enum PitchEvent {
    Ball,
    Strike,
    Foul,
}

/// The result of processing a [`PitchEvent`] against the current count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PitchOutcome {
    /// Count changed but the at-bat continues.
    CountUpdated,
    /// Four balls — a walk should be recorded.
    WalkForced,
    /// Three strikes — a strikeout should be recorded.
    StrikeoutForced,
}

/// The result of completing an at-bat, indicating whether the half-inning ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InningOutcome {
    /// Fewer than three outs; batting continues.
    Continue,
    /// Third out recorded; half-inning is over.
    ThreeOuts,
    /// Walk-off or regulation end; the game is over.
    GameOver,
}

// ── Error tracking ─────────────────────────────────────────────────────────

/// Fielding errors charged to each team in the current game.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameErrors {
    pub away: u8,
    pub home: u8,
}

// ── GameState ──────────────────────────────────────────────────────────────

/// The complete state of an in-progress or finished baseball game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub away: Team,
    pub home: Team,
    pub inning: u8,
    pub half: Half,
    pub outs: u8,
    pub count: Count,
    pub bases: Bases,
    pub inning_scores: Vec<InningScore>, // index = inning - 1
    pub play_log: Vec<PlayLogEntry>,
    pub errors: GameErrors,
    pub game_over: bool,
}

impl GameState {
    /// Creates a new game starting at the top of the first inning with a clean count and empty bases.
    pub fn new(away: Team, home: Team) -> Self {
        GameState {
            away,
            home,
            inning: 1,
            half: Half::Top,
            outs: 0,
            count: Count::default(),
            bases: Bases::default(),
            inning_scores: vec![InningScore::default()],
            play_log: Vec::new(),
            errors: GameErrors::default(),
            game_over: false,
        }
    }

    // ── Team accessors ─────────────────────────────────────────────────────

    /// Returns the team currently at bat (away in the top half, home in the bottom).
    pub fn batting_team(&self) -> &Team {
        if self.half == Half::Top {
            &self.away
        } else {
            &self.home
        }
    }

    /// Returns a mutable reference to the team currently at bat.
    pub fn batting_team_mut(&mut self) -> &mut Team {
        if self.half == Half::Top {
            &mut self.away
        } else {
            &mut self.home
        }
    }

    /// Returns the team currently in the field.
    pub fn fielding_team(&self) -> &Team {
        if self.half == Half::Top {
            &self.home
        } else {
            &self.away
        }
    }

    /// Returns a mutable reference to the team currently in the field.
    pub fn fielding_team_mut(&mut self) -> &mut Team {
        if self.half == Half::Top {
            &mut self.home
        } else {
            &mut self.away
        }
    }

    // ── Score totals ───────────────────────────────────────────────────────

    /// Returns the total runs scored by the away team across all innings.
    pub fn away_total_runs(&self) -> u8 {
        self.inning_scores.iter().map(|s| s.away_runs).sum()
    }

    /// Returns the total runs scored by the home team across all innings.
    pub fn home_total_runs(&self) -> u8 {
        self.inning_scores.iter().map(|s| s.home_runs).sum()
    }

    /// Returns the total hits recorded by the away team's lineup.
    pub fn away_total_hits(&self) -> u8 {
        self.away.total_hits()
    }
    /// Returns the total hits recorded by the home team's lineup.
    pub fn home_total_hits(&self) -> u8 {
        self.home.total_hits()
    }

    // ── Pitch processing ───────────────────────────────────────────────────

    /// Increments the fielding pitcher's pitch count and updates the count for the given event.
    ///
    /// Returns [`PitchOutcome::WalkForced`] on the fourth ball, [`PitchOutcome::StrikeoutForced`]
    /// on the third strike, or [`PitchOutcome::CountUpdated`] otherwise.
    pub fn apply_pitch(&mut self, event: PitchEvent) -> PitchOutcome {
        self.fielding_team_mut()
            .current_pitcher_mut()
            .stats
            .pitch_count += 1;
        match event {
            PitchEvent::Ball => {
                if self.count.add_ball() {
                    PitchOutcome::WalkForced
                } else {
                    PitchOutcome::CountUpdated
                }
            }
            PitchEvent::Strike => {
                if self.count.add_strike() {
                    PitchOutcome::StrikeoutForced
                } else {
                    PitchOutcome::CountUpdated
                }
            }
            PitchEvent::Foul => {
                self.count.add_foul();
                PitchOutcome::CountUpdated
            }
        }
    }

    // ── At-bat resolution ──────────────────────────────────────────────────

    /// Resolves a completed at-bat: updates all batter and pitcher stats, advances baserunners,
    /// records runs and errors, logs the play, resets the count, and checks for end-of-half-inning.
    ///
    /// `rbi` is the number of runs driven in by this at-bat (caller supplies this since it
    /// depends on baserunner positions at the time of the play).
    ///
    /// Returns [`InningOutcome::ThreeOuts`] when the side is retired,
    /// [`InningOutcome::GameOver`] on a walk-off or regulation end, or
    /// [`InningOutcome::Continue`] otherwise.
    pub fn apply_at_bat_result(&mut self, result: AtBatResult, rbi: u8) -> InningOutcome {
        // Snapshot values before mutable borrows
        let batter_idx = self.batting_team().batting_order_pos;
        let batter_name = self.batting_team().current_batter().info.name.clone();
        let description = result.display();
        let outs_this_play = result.records_out();
        let is_hit = result.is_hit();
        let counts_ab = result.counts_as_at_bat();
        let is_error = matches!(&result, AtBatResult::Error(_));

        // Update batter stats
        {
            let batter = self.batting_team_mut().current_batter_mut();
            if counts_ab {
                batter.stats.at_bats += 1;
            }
            if is_hit {
                batter.stats.hits += 1;
            }
            batter.stats.rbi += rbi;
            match &result {
                AtBatResult::Walk => batter.stats.walks += 1,
                AtBatResult::HitByPitch => batter.stats.hit_by_pitch += 1,
                AtBatResult::StrikeoutSwinging | AtBatResult::StrikeoutLooking => {
                    batter.stats.strikeouts += 1;
                }
                AtBatResult::Double => batter.stats.doubles += 1,
                AtBatResult::Triple => batter.stats.triples += 1,
                AtBatResult::HomeRun => batter.stats.home_runs += 1,
                AtBatResult::Error(_) => batter.stats.reached_on_error += 1,
                _ => {}
            }
        }

        // Update fielding pitcher stats
        {
            let pitcher = self.fielding_team_mut().current_pitcher_mut();
            pitcher.stats.outs_recorded += outs_this_play as u16;
            pitcher.stats.batters_faced += 1;
            if is_hit {
                pitcher.stats.hits_allowed += 1;
            }
            match &result {
                AtBatResult::Walk => pitcher.stats.walks += 1,
                AtBatResult::HitByPitch => pitcher.stats.hit_batsmen += 1,
                AtBatResult::StrikeoutSwinging | AtBatResult::StrikeoutLooking => {
                    pitcher.stats.strikeouts += 1;
                }
                _ => {}
            }
        }

        // Track fielding errors on the correct team
        if is_error {
            match self.half {
                Half::Top => self.errors.home += 1,
                Half::Bottom => self.errors.away += 1,
            }
        }

        // Advance bases, collect run scorers
        let (runs_scored, scorers) = self.advance_bases_for_result(&result, batter_idx);

        // Credit runs to the individual runners who scored
        for scorer_idx in &scorers {
            self.batting_team_mut().lineup[*scorer_idx].stats.runs += 1;
        }

        // Credit runs_allowed / earned_runs to the pitcher based on the runs
        // that actually crossed the plate on this play. All runs on an error
        // play are unearned; otherwise they are earned.
        {
            let pitcher = self.fielding_team_mut().current_pitcher_mut();
            pitcher.stats.runs_allowed += runs_scored;
            if !is_error {
                pitcher.stats.earned_runs += runs_scored;
            }
        }

        // Update inning score
        let inning_idx = (self.inning - 1) as usize;
        if inning_idx < self.inning_scores.len() {
            match self.half {
                Half::Top => self.inning_scores[inning_idx].away_runs += runs_scored,
                Half::Bottom => self.inning_scores[inning_idx].home_runs += runs_scored,
            }
        }

        // Advance batting order, log play, reset count
        self.batting_team_mut().advance_batter();
        self.play_log.push(PlayLogEntry {
            inning: self.inning,
            half: self.half.clone(),
            batter_name,
            description,
            rbi,
        });
        self.count.reset();

        // Check for end of half-inning
        self.outs += outs_this_play;
        if self.outs >= 3 {
            self.end_half_inning();
            if self.game_over {
                InningOutcome::GameOver
            } else {
                InningOutcome::ThreeOuts
            }
        } else {
            InningOutcome::Continue
        }
    }

    fn advance_bases_for_result(
        &mut self,
        result: &AtBatResult,
        batter_idx: usize,
    ) -> (u8, Vec<usize>) {
        match result {
            AtBatResult::Single => self.bases.advance_all(1, batter_idx),
            AtBatResult::Double => self.bases.advance_all(2, batter_idx),
            AtBatResult::Triple => self.bases.advance_all(3, batter_idx),
            AtBatResult::HomeRun => self.bases.advance_all(4, batter_idx),
            AtBatResult::Walk | AtBatResult::HitByPitch => self.bases.force_advance(batter_idx),
            AtBatResult::Error(_) => {
                // Batter reaches first. Forced runners advance (preserving the
                // runner originally on first, etc.); non-forced runners stay
                // in place. Bases-loaded errors score the runner from third.
                self.bases.force_advance(batter_idx)
            }
            AtBatResult::FieldersChoice => {
                self.bases.resolve_fielders_choice(batter_idx);
                (0, vec![])
            }
            AtBatResult::SacrificeFly(_) => {
                // Runner on third scores (tag-up)
                if let Some(idx) = self.bases.third.take() {
                    (1, vec![idx])
                } else {
                    (0, vec![])
                }
            }
            AtBatResult::DoublePlay(_) => {
                self.bases.resolve_double_play();
                (0, vec![])
            }
            AtBatResult::Groundout(_)
            | AtBatResult::Flyout(_)
            | AtBatResult::StrikeoutSwinging
            | AtBatResult::StrikeoutLooking => (0, vec![]),
        }
    }

    fn end_half_inning(&mut self) {
        // Count runners left on base before clearing
        let lob = self.bases.first.is_some() as u8
            + self.bases.second.is_some() as u8
            + self.bases.third.is_some() as u8;
        self.batting_team_mut().left_on_base += lob;

        self.bases.clear();
        self.outs = 0;
        self.count.reset();

        let was_bottom = self.half == Half::Bottom;
        self.half = match self.half {
            Half::Top => Half::Bottom,
            Half::Bottom => Half::Top,
        };

        if was_bottom {
            // Walk-off / regulation end: home leads after bottom of 9+
            if self.inning >= 9 && self.home_total_runs() > self.away_total_runs() {
                self.game_over = true;
                return;
            }
            self.inning += 1;
            self.inning_scores.push(InningScore::default());
        }
    }

    // ── Pitcher change ─────────────────────────────────────────────────────

    /// Adds a new pitcher to the fielding team's staff and makes them the current pitcher.
    pub fn change_pitcher(&mut self, new_pitcher: PitcherInfo) {
        let entered_inning = self.inning;
        let fielding = self.fielding_team_mut();
        fielding.pitchers.push(PitcherAppearance {
            info: new_pitcher,
            stats: PitcherGameStats::default(),
            entered_inning,
            decision: None,
        });
        fielding.current_pitcher_idx = fielding.pitchers.len() - 1;
    }

    // ── Batter change ──────────────────────────────────────────────────────

    /// Replaces the current batter with a new hitter in the same batting-order spot.
    pub fn change_batter(&mut self, new_batter: BatterInfo) {
        self.batting_team_mut().replace_current_batter(new_batter);
    }

    // ── Manual runner advance ──────────────────────────────────────────────

    /// Manually moves a runner between bases and updates the inning score if they score.
    ///
    /// `from`/`to` are 1-indexed; use `4` for home plate. Returns `true` if the runner scored.
    pub fn advance_runner(&mut self, from: u8, to: u8) -> bool {
        let scored = self.bases.move_runner(from, to);
        if scored {
            let inning_idx = (self.inning - 1) as usize;
            if inning_idx < self.inning_scores.len() {
                match self.half {
                    Half::Top => self.inning_scores[inning_idx].away_runs += 1,
                    Half::Bottom => self.inning_scores[inning_idx].home_runs += 1,
                }
            }
        }
        scored
    }

    // ── Stolen base / caught stealing ─────────────────────────────────────

    /// Credits a stolen base to `runner_idx` and logs the play.
    pub fn credit_stolen_base(&mut self, runner_idx: Option<usize>, to_base: u8) {
        if let Some(i) = runner_idx {
            let name = self.batting_team().lineup[i].info.name.clone();
            self.batting_team_mut().lineup[i].stats.stolen_bases += 1;
            let label = if to_base >= 4 {
                "H".to_string()
            } else {
                format!("{}B", to_base)
            };
            self.play_log.push(PlayLogEntry {
                inning: self.inning,
                half: self.half.clone(),
                batter_name: name,
                description: format!("SB (\u{2192}{})", label),
                rbi: 0,
            });
        }
    }

    /// Records a caught stealing at `to_base`: removes the runner (or undoes a scored run if
    /// `to_base == 4`), credits CS to `runner_idx`, increments outs, and checks for end-of-half.
    pub fn record_caught_stealing(
        &mut self,
        runner_idx: Option<usize>,
        to_base: u8,
    ) -> InningOutcome {
        if to_base <= 3 {
            match to_base {
                1 => {
                    self.bases.first = None;
                }
                2 => {
                    self.bases.second = None;
                }
                3 => {
                    self.bases.third = None;
                }
                _ => {}
            }
        } else {
            // Runner "scored" via advance_runner — undo that run from the line score
            let inning_idx = (self.inning - 1) as usize;
            if inning_idx < self.inning_scores.len() {
                match self.half {
                    Half::Top => {
                        self.inning_scores[inning_idx].away_runs =
                            self.inning_scores[inning_idx].away_runs.saturating_sub(1);
                    }
                    Half::Bottom => {
                        self.inning_scores[inning_idx].home_runs =
                            self.inning_scores[inning_idx].home_runs.saturating_sub(1);
                    }
                }
            }
        }
        if let Some(i) = runner_idx {
            let name = self.batting_team().lineup[i].info.name.clone();
            self.batting_team_mut().lineup[i].stats.caught_stealing += 1;
            let label = if to_base >= 4 {
                "H".to_string()
            } else {
                format!("{}B", to_base)
            };
            self.play_log.push(PlayLogEntry {
                inning: self.inning,
                half: self.half.clone(),
                batter_name: name,
                description: format!("CS (\u{2192}{})", label),
                rbi: 0,
            });
        }
        self.outs += 1;
        if self.outs >= 3 {
            self.end_half_inning();
            if self.game_over {
                InningOutcome::GameOver
            } else {
                InningOutcome::ThreeOuts
            }
        } else {
            InningOutcome::Continue
        }
    }

    // ── Decision assignment (call at game end) ─────────────────────────────

    /// Assigns win, loss, and save decisions to pitchers based on the final score.
    ///
    /// Should be called exactly once when `game_over` is set to `true`. If the winning
    /// team had two or more pitchers, the second-to-last gets the win and the last gets
    /// the save.
    pub fn assign_decisions(&mut self) {
        let home_won = self.home_total_runs() > self.away_total_runs();

        if home_won {
            let len = self.home.pitchers.len();
            if len == 1 {
                self.home.pitchers[0].decision = Some(Decision::Win);
            } else {
                self.home.pitchers[len - 2].decision = Some(Decision::Win);
                self.home.pitchers[len - 1].decision = Some(Decision::Save);
            }
            if let Some(last) = self.away.pitchers.last_mut() {
                last.decision = Some(Decision::Loss);
            }
        } else {
            let len = self.away.pitchers.len();
            if len == 1 {
                self.away.pitchers[0].decision = Some(Decision::Win);
            } else {
                self.away.pitchers[len - 2].decision = Some(Decision::Win);
                self.away.pitchers[len - 1].decision = Some(Decision::Save);
            }
            if let Some(last) = self.home.pitchers.last_mut() {
                last.decision = Some(Decision::Loss);
            }
        }
    }
}

// ── Unit tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::helpers::{make_game, make_team};

    // ── TeamColor ──────────────────────────────────────────────────────

    #[test]
    fn test_team_color_next_wraps() {
        assert_eq!(TeamColor::Red.next(), TeamColor::Orange);
        assert_eq!(TeamColor::White.next(), TeamColor::Red);
    }

    #[test]
    fn test_team_color_prev_wraps() {
        assert_eq!(TeamColor::Red.prev(), TeamColor::White);
        assert_eq!(TeamColor::Orange.prev(), TeamColor::Red);
    }

    #[test]
    fn test_team_color_default_is_white() {
        assert_eq!(TeamColor::default(), TeamColor::White);
    }

    #[test]
    fn test_team_color_name() {
        assert_eq!(TeamColor::Red.name(), "Red");
        assert_eq!(TeamColor::Orange.name(), "Orange");
        assert_eq!(TeamColor::Navy.name(), "Navy");
        assert_eq!(TeamColor::Brown.name(), "Brown");
    }

    #[test]
    fn test_team_color_full_cycle() {
        let start = TeamColor::Red;
        let mut c = start;
        for _ in 0..TeamColor::ALL.len() {
            c = c.next();
        }
        assert_eq!(c, start);
    }

    #[test]
    fn test_ip_display() {
        let mut s = PitcherGameStats::default();
        assert_eq!(s.ip_display(), "0.0");
        s.outs_recorded = 3;
        assert_eq!(s.ip_display(), "1.0");
        s.outs_recorded = 7;
        assert_eq!(s.ip_display(), "2.1");
        s.outs_recorded = 17;
        assert_eq!(s.ip_display(), "5.2");
    }

    #[test]
    fn test_walk_empty_bases() {
        let mut bases = Bases::default();
        let (runs, scorers) = bases.force_advance(0);
        assert_eq!(runs, 0);
        assert!(scorers.is_empty());
        assert_eq!(bases.first, Some(0));
        assert!(bases.second.is_none());
        assert!(bases.third.is_none());
    }

    #[test]
    fn test_walk_bases_loaded() {
        let mut bases = Bases {
            first: Some(0),
            second: Some(1),
            third: Some(2),
        };
        let (runs, scorers) = bases.force_advance(3);
        assert_eq!(runs, 1);
        assert_eq!(scorers, vec![2]);
        assert_eq!(bases.first, Some(3));
        assert_eq!(bases.second, Some(0));
        assert_eq!(bases.third, Some(1));
    }

    #[test]
    fn test_hr_clears_bases() {
        let mut bases = Bases {
            first: Some(0),
            second: Some(1),
            third: Some(2),
        };
        let (runs, _) = bases.advance_all(4, 3);
        assert_eq!(runs, 4);
        assert!(bases.first.is_none());
        assert!(bases.second.is_none());
        assert!(bases.third.is_none());
    }

    #[test]
    fn test_single_advancement() {
        let mut bases = Bases {
            first: Some(0),
            second: None,
            third: Some(2),
        };
        let (runs, _) = bases.advance_all(1, 3);
        assert_eq!(runs, 1); // runner from third scores
        assert_eq!(bases.first, Some(3)); // batter
        assert_eq!(bases.second, Some(0)); // old first
        assert!(bases.third.is_none());
    }

    #[test]
    fn test_double_play_records_two_outs() {
        assert_eq!(AtBatResult::DoublePlay(vec![6, 4, 3]).records_out(), 2);
    }

    #[test]
    fn test_dp_applied_in_game() {
        let mut game = make_game();
        game.bases.first = Some(0);
        let outcome = game.apply_at_bat_result(AtBatResult::DoublePlay(vec![6, 4, 3]), 0);
        assert_eq!(game.outs, 2);
        assert!(matches!(outcome, InningOutcome::Continue));
        assert!(game.bases.first.is_none());
    }

    #[test]
    fn test_three_outs_end_half() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        let outcome = game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        assert!(matches!(outcome, InningOutcome::ThreeOuts));
        assert_eq!(game.half, Half::Bottom);
        assert_eq!(game.outs, 0);
        assert!(game.bases.is_empty());
    }

    #[test]
    fn test_score_accumulates() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::HomeRun, 1);
        assert_eq!(game.away_total_runs(), 1);
        assert_eq!(game.inning_scores[0].away_runs, 1);
    }

    #[test]
    fn test_batting_order_wraps() {
        // Apply 9 singles (no outs) so all 9 away batters cycle through.
        // After spot 8 advance_batter wraps back to 0.
        let mut game = make_game();
        for _ in 0..9 {
            game.apply_at_bat_result(AtBatResult::Single, 0);
        }
        assert_eq!(game.away.batting_order_pos, 0);
    }

    #[test]
    fn test_count_reset_after_at_bat() {
        let mut game = make_game();
        game.apply_pitch(PitchEvent::Ball);
        game.apply_pitch(PitchEvent::Ball);
        game.apply_at_bat_result(AtBatResult::Single, 0);
        assert_eq!(game.count.balls, 0);
        assert_eq!(game.count.strikes, 0);
    }

    // ── Count ──────────────────────────────────────────────────────────────

    #[test]
    fn test_count_foul_no_third_strike() {
        let mut c = Count {
            balls: 0,
            strikes: 2,
        };
        c.add_foul();
        assert_eq!(c.strikes, 2);
    }

    #[test]
    fn test_count_foul_advances_below_two() {
        let mut c = Count::default();
        c.add_foul();
        assert_eq!(c.strikes, 1);
        c.add_foul();
        assert_eq!(c.strikes, 2);
        // third foul should not advance
        c.add_foul();
        assert_eq!(c.strikes, 2);
    }

    #[test]
    fn test_count_ball_walk_at_four() {
        let mut c = Count {
            balls: 3,
            strikes: 0,
        };
        assert!(c.add_ball());
        assert_eq!(c.balls, 4);
    }

    #[test]
    fn test_count_ball_no_walk_below_four() {
        let mut c = Count::default();
        assert!(!c.add_ball());
        assert!(!c.add_ball());
        assert!(!c.add_ball());
    }

    #[test]
    fn test_count_strike_so_at_three() {
        let mut c = Count {
            balls: 0,
            strikes: 2,
        };
        assert!(c.add_strike());
    }

    #[test]
    fn test_count_strike_no_so_below_three() {
        let mut c = Count::default();
        assert!(!c.add_strike());
        assert!(!c.add_strike());
    }

    #[test]
    fn test_count_reset_clears() {
        let mut c = Count {
            balls: 3,
            strikes: 2,
        };
        c.reset();
        assert_eq!(c.balls, 0);
        assert_eq!(c.strikes, 0);
    }

    // ── AtBatResult predicates ─────────────────────────────────────────────

    #[test]
    fn test_at_bat_result_is_hit_variants() {
        assert!(AtBatResult::Single.is_hit());
        assert!(AtBatResult::Double.is_hit());
        assert!(AtBatResult::Triple.is_hit());
        assert!(AtBatResult::HomeRun.is_hit());
    }

    #[test]
    fn test_at_bat_result_non_hits() {
        assert!(!AtBatResult::Walk.is_hit());
        assert!(!AtBatResult::StrikeoutSwinging.is_hit());
        assert!(!AtBatResult::Groundout(vec![6, 3]).is_hit());
        assert!(!AtBatResult::Error(6).is_hit());
        assert!(!AtBatResult::FieldersChoice.is_hit());
    }

    #[test]
    fn test_records_out_variants() {
        assert_eq!(AtBatResult::Single.records_out(), 0);
        assert_eq!(AtBatResult::Walk.records_out(), 0);
        assert_eq!(AtBatResult::StrikeoutSwinging.records_out(), 1);
        assert_eq!(AtBatResult::StrikeoutLooking.records_out(), 1);
        assert_eq!(AtBatResult::Groundout(vec![6, 3]).records_out(), 1);
        assert_eq!(AtBatResult::Flyout(8).records_out(), 1);
        assert_eq!(AtBatResult::FieldersChoice.records_out(), 1);
        assert_eq!(AtBatResult::SacrificeFly(9).records_out(), 1);
        assert_eq!(AtBatResult::DoublePlay(vec![6, 4, 3]).records_out(), 2);
    }

    #[test]
    fn test_counts_as_at_bat_variants() {
        assert!(AtBatResult::Single.counts_as_at_bat());
        assert!(AtBatResult::StrikeoutSwinging.counts_as_at_bat());
        assert!(AtBatResult::Groundout(vec![6, 3]).counts_as_at_bat());
        assert!(!AtBatResult::Walk.counts_as_at_bat());
        assert!(!AtBatResult::HitByPitch.counts_as_at_bat());
        assert!(!AtBatResult::SacrificeFly(9).counts_as_at_bat());
    }

    // ── AtBatResult::display ───────────────────────────────────────────────

    #[test]
    fn test_at_bat_result_display_all_variants() {
        assert_eq!(AtBatResult::Single.display(), "1B");
        assert_eq!(AtBatResult::Double.display(), "2B");
        assert_eq!(AtBatResult::Triple.display(), "3B");
        assert_eq!(AtBatResult::HomeRun.display(), "HR");
        assert_eq!(AtBatResult::StrikeoutSwinging.display(), "K");
        assert_eq!(AtBatResult::StrikeoutLooking.display(), "Kl");
        assert_eq!(AtBatResult::Walk.display(), "BB");
        assert_eq!(AtBatResult::HitByPitch.display(), "HBP");
        assert_eq!(AtBatResult::Groundout(vec![6, 3]).display(), "6-3");
        assert_eq!(AtBatResult::DoublePlay(vec![6, 4, 3]).display(), "6-4-3 DP");
        assert_eq!(AtBatResult::Flyout(8).display(), "F8");
        assert_eq!(AtBatResult::Error(6).display(), "E6");
        assert_eq!(AtBatResult::FieldersChoice.display(), "FC");
        assert_eq!(AtBatResult::SacrificeFly(9).display(), "SF9");
    }

    // ── BatterInfo / PitcherGameStats display ──────────────────────────────

    #[test]
    fn test_avg_display_zero() {
        let b = BatterInfo {
            name: "X".into(),
            season_avg: 0.0,
        };
        assert_eq!(b.avg_display(), ".---");
    }

    #[test]
    fn test_avg_display_negative() {
        let b = BatterInfo {
            name: "X".into(),
            season_avg: -0.1,
        };
        assert_eq!(b.avg_display(), ".---");
    }

    #[test]
    fn test_avg_display_normal() {
        let b = BatterInfo {
            name: "X".into(),
            season_avg: 0.315,
        };
        assert_eq!(b.avg_display(), ".315");
    }

    #[test]
    fn test_decision_label() {
        assert_eq!(Decision::Win.label(), "W");
        assert_eq!(Decision::Loss.label(), "L");
        assert_eq!(Decision::Save.label(), "S");
    }

    #[test]
    fn test_play_log_entry_display_top() {
        let entry = PlayLogEntry {
            inning: 3,
            half: Half::Top,
            batter_name: "Smith".into(),
            description: "1B".into(),
            rbi: 1,
        };
        let s = entry.display();
        assert!(s.contains("T3"));
        assert!(s.contains("Smith"));
        assert!(s.contains("1B"));
        assert!(s.contains("1 RBI"));
    }

    #[test]
    fn test_play_log_entry_display_bottom() {
        let entry = PlayLogEntry {
            inning: 7,
            half: Half::Bottom,
            batter_name: "Jones".into(),
            description: "HR".into(),
            rbi: 2,
        };
        let s = entry.display();
        assert!(s.starts_with("B7"));
    }

    // ── Bases::advance_all ─────────────────────────────────────────────────

    #[test]
    fn test_advance_all_single_no_runners() {
        let mut bases = Bases::default();
        let (runs, scorers) = bases.advance_all(1, 0);
        assert_eq!(runs, 0);
        assert!(scorers.is_empty());
        assert_eq!(bases.first, Some(0));
    }

    #[test]
    fn test_advance_all_double_runner_on_first() {
        let mut bases = Bases {
            first: Some(0),
            second: None,
            third: None,
        };
        let (runs, _) = bases.advance_all(2, 1);
        assert_eq!(runs, 0);
        assert_eq!(bases.second, Some(1)); // batter on 2nd
        assert_eq!(bases.third, Some(0)); // old runner on 3rd
        assert!(bases.first.is_none());
    }

    #[test]
    fn test_advance_all_triple_runner_on_second() {
        let mut bases = Bases {
            first: None,
            second: Some(0),
            third: None,
        };
        let (runs, scorers) = bases.advance_all(3, 1);
        assert_eq!(runs, 1);
        assert_eq!(scorers, vec![0]);
        assert_eq!(bases.third, Some(1)); // batter on 3rd
    }

    #[test]
    fn test_advance_all_hr_solo() {
        let mut bases = Bases::default();
        let (runs, scorers) = bases.advance_all(4, 0);
        assert_eq!(runs, 1);
        assert_eq!(scorers, vec![0]);
        assert!(bases.first.is_none());
    }

    // ── Bases::force_advance ───────────────────────────────────────────────

    #[test]
    fn test_force_advance_runner_on_first_only() {
        let mut bases = Bases {
            first: Some(0),
            second: None,
            third: None,
        };
        let (runs, scorers) = bases.force_advance(1);
        assert_eq!(runs, 0);
        assert!(scorers.is_empty());
        assert_eq!(bases.first, Some(1));
        assert_eq!(bases.second, Some(0));
        assert!(bases.third.is_none());
    }

    #[test]
    fn test_force_advance_runners_on_first_second() {
        let mut bases = Bases {
            first: Some(0),
            second: Some(1),
            third: None,
        };
        let (runs, scorers) = bases.force_advance(2);
        assert_eq!(runs, 0);
        assert!(scorers.is_empty());
        assert_eq!(bases.first, Some(2));
        assert_eq!(bases.second, Some(0));
        assert_eq!(bases.third, Some(1));
    }

    // ── Bases::move_runner ─────────────────────────────────────────────────

    #[test]
    fn test_move_runner_1_to_2() {
        let mut bases = Bases {
            first: Some(0),
            second: None,
            third: None,
        };
        assert!(!bases.move_runner(1, 2));
        assert!(bases.first.is_none());
        assert_eq!(bases.second, Some(0));
    }

    #[test]
    fn test_move_runner_3_to_home() {
        let mut bases = Bases {
            first: None,
            second: None,
            third: Some(0),
        };
        assert!(bases.move_runner(3, 4));
        assert!(bases.third.is_none());
    }

    #[test]
    fn test_move_runner_from_empty_base() {
        let mut bases = Bases::default();
        assert!(!bases.move_runner(1, 2));
    }

    // ── GameState at-bat resolution ────────────────────────────────────────

    #[test]
    fn test_walk_empty_bases_no_run() {
        let mut game = make_game();
        let io = game.apply_at_bat_result(AtBatResult::Walk, 0);
        assert_eq!(game.away_total_runs(), 0);
        assert_eq!(game.bases.first, Some(0));
        assert!(matches!(io, InningOutcome::Continue));
    }

    #[test]
    fn test_walk_bases_loaded_run_scores() {
        let mut game = make_game();
        game.bases = Bases {
            first: Some(0),
            second: Some(1),
            third: Some(2),
        };
        game.apply_at_bat_result(AtBatResult::Walk, 1);
        assert_eq!(game.away_total_runs(), 1);
    }

    #[test]
    fn test_hbp_empty_bases() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::HitByPitch, 0);
        assert_eq!(game.bases.first, Some(0));
        assert_eq!(game.away.current_pitcher().stats.hit_batsmen, 0);
        assert_eq!(game.home.current_pitcher().stats.hit_batsmen, 1);
    }

    #[test]
    fn test_single_runner_scores_from_third() {
        let mut game = make_game();
        game.bases.third = Some(2);
        game.apply_at_bat_result(AtBatResult::Single, 1);
        assert_eq!(game.away_total_runs(), 1);
    }

    #[test]
    fn test_error_batter_reaches_no_advance() {
        let mut game = make_game();
        game.bases.second = Some(1);
        game.apply_at_bat_result(AtBatResult::Error(6), 0);
        // batter on first, runner on second stays (not forced)
        assert_eq!(game.bases.first, Some(0));
        assert_eq!(game.bases.second, Some(1));
        assert_eq!(game.errors.home, 1); // top half → home team commits error
    }

    #[test]
    fn test_error_runner_on_first_is_preserved() {
        // Regression: a runner on first used to be overwritten when the batter
        // was placed on first after an Error.
        let mut game = make_game();
        game.bases.first = Some(5);
        game.apply_at_bat_result(AtBatResult::Error(6), 0);
        // Batter (index 0) takes first; runner originally on first is forced to second.
        assert_eq!(game.bases.first, Some(0));
        assert_eq!(game.bases.second, Some(5));
        assert!(game.bases.third.is_none());
        assert_eq!(game.outs, 0);
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_error_runners_on_first_and_second_shift_forward() {
        let mut game = make_game();
        game.bases.first = Some(4);
        game.bases.second = Some(5);
        game.apply_at_bat_result(AtBatResult::Error(6), 0);
        assert_eq!(game.bases.first, Some(0));
        assert_eq!(game.bases.second, Some(4));
        assert_eq!(game.bases.third, Some(5));
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_error_bases_loaded_scores_runner_from_third_unearned() {
        let mut game = make_game();
        game.bases.first = Some(3);
        game.bases.second = Some(4);
        game.bases.third = Some(5);
        game.apply_at_bat_result(AtBatResult::Error(6), 0);
        // Bases stay loaded with new runners; runner from third scores.
        assert_eq!(game.bases.first, Some(0));
        assert_eq!(game.bases.second, Some(3));
        assert_eq!(game.bases.third, Some(4));
        assert_eq!(game.away_total_runs(), 1);
        // Pitcher is charged with a run, but it is unearned on an error.
        assert_eq!(game.home.current_pitcher().stats.runs_allowed, 1);
        assert_eq!(game.home.current_pitcher().stats.earned_runs, 0);
        // The scoring runner gets credited with a run.
        assert_eq!(game.away.lineup[5].stats.runs, 1);
    }

    #[test]
    fn test_pitcher_runs_allowed_tracks_actual_runs_not_rbi() {
        // On a bases-loaded single, runs_scored=1 (runner from 3rd) even if
        // the caller happens to pass a different rbi value. Pitcher stats
        // should reflect the actual run, not the rbi parameter.
        let mut game = make_game();
        game.bases.third = Some(5);
        // Intentionally pass rbi=0 even though a run scored.
        game.apply_at_bat_result(AtBatResult::Single, 0);
        assert_eq!(game.away_total_runs(), 1);
        assert_eq!(game.home.current_pitcher().stats.runs_allowed, 1);
        assert_eq!(game.home.current_pitcher().stats.earned_runs, 1);
    }

    #[test]
    fn test_error_runner_on_third_only_stays_put() {
        // Runner on third is not forced by the batter taking first.
        let mut game = make_game();
        game.bases.third = Some(2);
        game.apply_at_bat_result(AtBatResult::Error(6), 0);
        assert_eq!(game.bases.first, Some(0));
        assert_eq!(game.bases.third, Some(2));
        assert!(game.bases.second.is_none());
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_fielders_choice_lead_runner_out() {
        let mut game = make_game();
        // Batter 0 singles, runner 0 is now on first
        game.apply_at_bat_result(AtBatResult::Single, 0);
        assert_eq!(game.bases.first, Some(0));
        // Batter 1 hits FC: runner 0 is retired, batter 1 (index 1) reaches first
        game.apply_at_bat_result(AtBatResult::FieldersChoice, 0);
        assert_eq!(game.bases.first, Some(1));
        assert!(game.bases.second.is_none());
        assert_eq!(game.outs, 1);
    }

    #[test]
    fn test_fielders_choice_first_and_second_retires_lead_forced_runner() {
        // Forced lead runner is the runner on second (forced to third).
        let mut game = make_game();
        game.bases.first = Some(4);
        game.bases.second = Some(5);
        game.apply_at_bat_result(AtBatResult::FieldersChoice, 0);
        assert_eq!(game.bases.first, Some(0)); // batter
        assert_eq!(game.bases.second, Some(4)); // runner from first advances
        assert!(game.bases.third.is_none()); // runner from second retired at third
        assert_eq!(game.outs, 1);
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_fielders_choice_first_and_third_retires_runner_from_first() {
        // Runner on third is not forced; runner on first is forced to second and retired.
        let mut game = make_game();
        game.bases.first = Some(4);
        game.bases.third = Some(5);
        game.apply_at_bat_result(AtBatResult::FieldersChoice, 0);
        assert_eq!(game.bases.first, Some(0)); // batter
        assert!(game.bases.second.is_none()); // runner from first retired at second
        assert_eq!(game.bases.third, Some(5)); // runner on third unchanged
        assert_eq!(game.outs, 1);
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_fielders_choice_bases_loaded_retires_runner_at_home() {
        // Bases loaded: runner from third is the lead forced runner (forced at home).
        // The retired runner does NOT score.
        let mut game = make_game();
        game.bases.first = Some(3);
        game.bases.second = Some(4);
        game.bases.third = Some(5);
        game.apply_at_bat_result(AtBatResult::FieldersChoice, 0);
        assert_eq!(game.bases.first, Some(0));
        assert_eq!(game.bases.second, Some(3));
        assert_eq!(game.bases.third, Some(4));
        assert_eq!(game.outs, 1);
        assert_eq!(game.away_total_runs(), 0);
        assert_eq!(game.away.lineup[5].stats.runs, 0);
    }

    #[test]
    fn test_fielders_choice_second_only_retires_runner_from_second() {
        // No force: lead runner on second is retired.
        let mut game = make_game();
        game.bases.second = Some(4);
        game.apply_at_bat_result(AtBatResult::FieldersChoice, 0);
        assert_eq!(game.bases.first, Some(0));
        assert!(game.bases.second.is_none());
        assert!(game.bases.third.is_none());
        assert_eq!(game.outs, 1);
    }

    #[test]
    fn test_fielders_choice_third_only_retires_runner_from_third() {
        let mut game = make_game();
        game.bases.third = Some(4);
        game.apply_at_bat_result(AtBatResult::FieldersChoice, 0);
        assert_eq!(game.bases.first, Some(0));
        assert!(game.bases.third.is_none());
        assert_eq!(game.outs, 1);
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_double_play_bases_runner_on_first_retired() {
        let mut game = make_game();
        game.bases.first = Some(5);
        let io = game.apply_at_bat_result(AtBatResult::DoublePlay(vec![6, 4, 3]), 0);
        assert_eq!(game.outs, 2);
        assert!(matches!(io, InningOutcome::Continue));
        // Both the runner from first and the batter are out; bases empty.
        assert!(game.bases.first.is_none());
        assert!(game.bases.second.is_none());
        assert!(game.bases.third.is_none());
    }

    #[test]
    fn test_double_play_first_and_second_keeps_runner_on_second() {
        // Runner forced out at 2nd; runner originally on 2nd stays at 2nd on a
        // conservative scoring interpretation.
        let mut game = make_game();
        game.bases.first = Some(4);
        game.bases.second = Some(5);
        game.apply_at_bat_result(AtBatResult::DoublePlay(vec![6, 4, 3]), 0);
        assert!(game.bases.first.is_none());
        assert_eq!(game.bases.second, Some(5));
        assert!(game.bases.third.is_none());
        assert_eq!(game.outs, 2);
    }

    #[test]
    fn test_double_play_bases_loaded_keeps_runners_on_second_and_third() {
        let mut game = make_game();
        game.bases.first = Some(3);
        game.bases.second = Some(4);
        game.bases.third = Some(5);
        game.apply_at_bat_result(AtBatResult::DoublePlay(vec![6, 4, 3]), 0);
        // Runner forced from first is out; batter is out; other runners stay.
        // RBI is not automatically credited on a DP.
        assert!(game.bases.first.is_none());
        assert_eq!(game.bases.second, Some(4));
        assert_eq!(game.bases.third, Some(5));
        assert_eq!(game.outs, 2);
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_double_play_no_runner_on_first_retires_lead_runner() {
        // Line-drive style DP with runners on 2nd and 3rd: lead runner doubled off.
        let mut game = make_game();
        game.bases.second = Some(4);
        game.bases.third = Some(5);
        game.apply_at_bat_result(AtBatResult::DoublePlay(vec![8, 5]), 0);
        assert!(game.bases.first.is_none());
        assert_eq!(game.bases.second, Some(4));
        assert!(game.bases.third.is_none());
        assert_eq!(game.outs, 2);
    }

    #[test]
    fn test_double_play_batter_is_not_placed_on_base() {
        let mut game = make_game();
        game.bases.first = Some(5);
        game.apply_at_bat_result(AtBatResult::DoublePlay(vec![6, 4, 3]), 0);
        assert!(game.bases.first.is_none());
        // Batting order still advances regardless.
        assert_eq!(game.away.batting_order_pos, 1);
    }

    // ── Bases::resolve_fielders_choice direct tests ────────────────────────

    #[test]
    fn test_resolve_fielders_choice_empty_bases_no_retirement() {
        // With empty bases, no runner is retired AND the batter is not placed
        // on base; the single FC out is attributed to the batter.
        let mut bases = Bases::default();
        assert!(!bases.resolve_fielders_choice(7));
        assert!(bases.first.is_none());
        assert!(bases.second.is_none());
        assert!(bases.third.is_none());
    }

    #[test]
    fn test_fielders_choice_empty_bases_treats_batter_as_out() {
        // FC with empty bases is a degenerate UI selection; the state must
        // stay consistent: bases empty, outs += 1, no runs.
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::FieldersChoice, 0);
        assert!(game.bases.first.is_none());
        assert!(game.bases.second.is_none());
        assert!(game.bases.third.is_none());
        assert_eq!(game.outs, 1);
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_resolve_fielders_choice_first_only() {
        let mut bases = Bases {
            first: Some(4),
            second: None,
            third: None,
        };
        assert!(bases.resolve_fielders_choice(0));
        assert_eq!(bases.first, Some(0));
        assert!(bases.second.is_none());
    }

    #[test]
    fn test_resolve_double_play_first_only() {
        let mut bases = Bases {
            first: Some(4),
            second: None,
            third: None,
        };
        assert!(bases.resolve_double_play());
        assert!(bases.first.is_none());
    }

    #[test]
    fn test_resolve_double_play_empty() {
        let mut bases = Bases::default();
        assert!(!bases.resolve_double_play());
    }

    #[test]
    fn test_sac_fly_runner_on_third_scores() {
        let mut game = make_game();
        game.bases.third = Some(2);
        let io = game.apply_at_bat_result(AtBatResult::SacrificeFly(9), 1);
        assert_eq!(game.away_total_runs(), 1);
        assert!(game.bases.third.is_none());
        assert!(matches!(io, InningOutcome::Continue));
    }

    #[test]
    fn test_sac_fly_no_runner_on_third() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::SacrificeFly(9), 0);
        assert_eq!(game.away_total_runs(), 0);
        assert_eq!(game.outs, 1);
    }

    #[test]
    fn test_groundout_no_base_change() {
        let mut game = make_game();
        game.bases.second = Some(1);
        game.apply_at_bat_result(AtBatResult::Groundout(vec![6, 3]), 0);
        assert_eq!(game.bases.second, Some(1));
        assert_eq!(game.outs, 1);
    }

    // ── GameState: game flow ───────────────────────────────────────────────

    #[test]
    fn test_end_half_inning_top_to_bottom() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        assert_eq!(game.half, Half::Bottom);
        assert_eq!(game.inning, 1);
        assert_eq!(game.outs, 0);
    }

    #[test]
    fn test_end_half_inning_bottom_advances_inning() {
        let mut game = make_game();
        // top half: 3 outs
        for _ in 0..3 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        // bottom half: 3 outs
        for _ in 0..3 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        assert_eq!(game.inning, 2);
        assert_eq!(game.half, Half::Top);
    }

    #[test]
    fn test_game_not_over_before_ninth() {
        let mut game = make_game();
        // Play through 8 full innings (3 outs per half)
        for _ in 0..16 {
            for _ in 0..3 {
                game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
            }
        }
        assert_eq!(game.inning, 9);
        assert!(!game.game_over);
    }

    #[test]
    fn test_game_over_after_bottom_ninth_home_wins() {
        // The game ends at the 3-out boundary when home leads after the bottom half of inning 9+.
        let mut game = make_game();
        // Play 8 full scoreless innings
        for _ in 0..16 {
            for _ in 0..3 {
                game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
            }
        }
        assert_eq!(game.inning, 9);
        assert_eq!(game.half, Half::Top);
        // Top 9: away scores 0, 3 Ks
        for _ in 0..3 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        // Bottom 9: home hits a HR (1 run), then three outs → game ends
        game.apply_at_bat_result(AtBatResult::HomeRun, 1);
        for _ in 0..2 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        let outcome = game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        assert!(matches!(outcome, InningOutcome::GameOver));
        assert!(game.game_over);
        assert_eq!(game.home_total_runs(), 1);
        assert_eq!(game.away_total_runs(), 0);
    }

    #[test]
    fn test_bases_cleared_after_half_inning() {
        let mut game = make_game();
        game.bases.first = Some(0);
        game.bases.second = Some(1);
        for _ in 0..3 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        assert!(game.bases.is_empty());
    }

    // ── GameState: pitchers and stats ──────────────────────────────────────

    #[test]
    fn test_pitch_count_increments() {
        let mut game = make_game();
        game.apply_pitch(PitchEvent::Ball);
        game.apply_pitch(PitchEvent::Strike);
        game.apply_pitch(PitchEvent::Foul);
        assert_eq!(game.home.current_pitcher().stats.pitch_count, 3);
    }

    #[test]
    fn test_pitcher_outs_recorded() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        game.apply_at_bat_result(AtBatResult::Groundout(vec![6, 3]), 0);
        assert_eq!(game.home.current_pitcher().stats.outs_recorded, 2);
    }

    #[test]
    fn test_batter_stats_accumulate() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::Single, 1);
        let batter = &game.away.lineup[0].stats;
        assert_eq!(batter.at_bats, 1);
        assert_eq!(batter.hits, 1);
        assert_eq!(batter.rbi, 1);
    }

    #[test]
    fn test_batter_walk_not_counted_as_ab() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::Walk, 0);
        assert_eq!(game.away.lineup[0].stats.at_bats, 0);
        assert_eq!(game.away.lineup[0].stats.walks, 1);
    }

    #[test]
    fn test_change_batter_preserves_replaced_hitter_stats() {
        let mut game = make_game();
        game.away.lineup[0].stats.at_bats = 1;
        game.away.lineup[0].stats.hits = 1;

        game.change_batter(BatterInfo {
            name: "Pinch Hitter".into(),
            season_avg: 0.333,
        });
        game.apply_at_bat_result(AtBatResult::Walk, 0);

        assert_eq!(game.away.substitutions.len(), 1);
        assert_eq!(game.away.substitutions[0].slot.info.name, "Player 1");
        assert_eq!(game.away.substitutions[0].slot.stats.hits, 1);
        assert_eq!(game.away.lineup[0].info.name, "Pinch Hitter");
        assert_eq!(game.away.lineup[0].stats.walks, 1);
        assert_eq!(game.away.lineup[0].stats.hits, 0);
        assert_eq!(game.away_total_hits(), 1);
    }

    #[test]
    fn test_batting_rows_include_substituted_hitters_in_order() {
        let mut team = make_team("Away");
        team.batting_order_pos = 2;
        team.replace_current_batter(BatterInfo {
            name: "Bench Bat".into(),
            season_avg: 0.290,
        });

        let names: Vec<&str> = team
            .batting_rows()
            .into_iter()
            .map(|slot| slot.info.name.as_str())
            .collect();

        assert_eq!(names.len(), 10);
        assert_eq!(names[0], "Player 1");
        assert_eq!(names[1], "Player 2");
        assert_eq!(names[2], "Player 3");
        assert_eq!(names[3], "Bench Bat");
    }

    #[test]
    fn test_change_pitcher_sets_current() {
        let mut game = make_game();
        game.change_pitcher(PitcherInfo {
            name: "Reliever".into(),
        });
        assert_eq!(game.home.current_pitcher().info.name, "Reliever");
        assert_eq!(game.home.pitchers.len(), 2);
        assert_eq!(game.home.current_pitcher().entered_inning, 1);
    }

    #[test]
    fn test_assign_decisions_home_wins_solo_pitcher() {
        let mut game = make_game();
        // Home scores in bottom 1st
        for _ in 0..3 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        game.apply_at_bat_result(AtBatResult::HomeRun, 1);
        game.assign_decisions();
        assert_eq!(game.home.pitchers[0].decision, Some(Decision::Win));
        assert_eq!(game.away.pitchers[0].decision, Some(Decision::Loss));
    }

    #[test]
    fn test_assign_decisions_away_wins() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::HomeRun, 1);
        game.assign_decisions();
        assert_eq!(game.away.pitchers[0].decision, Some(Decision::Win));
        assert_eq!(game.home.pitchers[0].decision, Some(Decision::Loss));
    }

    #[test]
    fn test_assign_decisions_with_save() {
        let mut game = make_game();
        // Away scores in top of 1st
        game.apply_at_bat_result(AtBatResult::HomeRun, 1);
        for _ in 0..3 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        // Now bottom of 1st — away is the fielding team; add a reliever to away
        assert_eq!(game.half, Half::Bottom);
        game.change_pitcher(PitcherInfo {
            name: "Closer".into(),
        });
        game.assign_decisions();
        // Away won with 2 pitchers: first gets Win, second gets Save
        assert_eq!(game.away.pitchers[0].decision, Some(Decision::Win));
        assert_eq!(game.away.pitchers[1].decision, Some(Decision::Save));
        assert_eq!(game.home.pitchers[0].decision, Some(Decision::Loss));
    }

    // ── Inning scoring ─────────────────────────────────────────────────────

    #[test]
    fn test_inning_scores_tracked_per_inning() {
        let mut game = make_game();
        game.apply_at_bat_result(AtBatResult::HomeRun, 1);
        // Move to inning 2
        for _ in 0..3 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        for _ in 0..3 {
            game.apply_at_bat_result(AtBatResult::StrikeoutSwinging, 0);
        }
        game.apply_at_bat_result(AtBatResult::HomeRun, 1);
        assert_eq!(game.inning_scores[0].away_runs, 1);
        assert_eq!(game.inning_scores[1].away_runs, 1);
    }

    #[test]
    fn test_manual_advance_runner_scores() {
        let mut game = make_game();
        game.bases.third = Some(0);
        let scored = game.advance_runner(3, 4);
        assert!(scored);
        assert_eq!(game.away_total_runs(), 1);
        assert!(game.bases.third.is_none());
    }
}
