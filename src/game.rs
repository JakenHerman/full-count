use serde::{Deserialize, Serialize};

// ── Half-inning direction ──────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Half {
    Top,
    Bottom,
}

// ── At-bat result ──────────────────────────────────────────────────────────

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
    Groundout(Vec<u8>),     // fielder sequence e.g. [6,3] or [5,4,3]
    DoublePlay(Vec<u8>),    // e.g. [6,4,3] — records 2 outs
    Flyout(u8),             // single fielder position
    Error(u8),              // fielder who made the error
    FieldersChoice,
    SacrificeFly(u8),
}

impl AtBatResult {
    pub fn is_hit(&self) -> bool {
        matches!(
            self,
            AtBatResult::Single | AtBatResult::Double | AtBatResult::Triple | AtBatResult::HomeRun
        )
    }

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

    pub fn counts_as_at_bat(&self) -> bool {
        !matches!(
            self,
            AtBatResult::Walk | AtBatResult::HitByPitch | AtBatResult::SacrificeFly(_)
        )
    }

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
    seq.iter().map(|n| n.to_string()).collect::<Vec<_>>().join("-")
}

// ── Game stats ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BatterGameStats {
    pub at_bats: u8,
    pub runs: u8,
    pub hits: u8,
    pub rbi: u8,
    pub walks: u8,
    pub strikeouts: u8,
    pub hit_by_pitch: u8,
}

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
}

impl PitcherGameStats {
    pub fn ip_display(&self) -> String {
        format!("{}.{}", self.outs_recorded / 3, self.outs_recorded % 3)
    }
}

// ── Roster types ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatterInfo {
    pub name: String,
    pub season_avg: f32,
}

impl BatterInfo {
    pub fn avg_display(&self) -> String {
        if self.season_avg <= 0.0 {
            ".---".to_string()
        } else {
            format!(".{:03}", (self.season_avg * 1000.0).round() as u32)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PitcherInfo {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineupSlot {
    pub info: BatterInfo,
    pub stats: BatterGameStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Win,
    Loss,
    Save,
}

impl Decision {
    pub fn label(&self) -> &'static str {
        match self {
            Decision::Win => "W",
            Decision::Loss => "L",
            Decision::Save => "S",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PitcherAppearance {
    pub info: PitcherInfo,
    pub stats: PitcherGameStats,
    pub entered_inning: u8,
    pub decision: Option<Decision>,
}

// ── Bases ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bases {
    pub first: Option<usize>,   // lineup index of runner on base
    pub second: Option<usize>,
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

    /// Advance all runners by `bases` bases (1=single, 2=double, 3=triple, 4=HR).
    /// Returns (runs_scored, lineup_indices_who_scored).
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

    /// Force-advance for walk/HBP: only advance runners who are forced.
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

    /// Move a specific runner from one base to another.
    /// from/to: 1=1st, 2=2nd, 3=3rd, 4=home (scores).
    /// Returns true if the runner scored.
    pub fn move_runner(&mut self, from: u8, to: u8) -> bool {
        let runner = match from {
            1 => self.first.take(),
            2 => self.second.take(),
            3 => self.third.take(),
            _ => return false,
        };
        if let Some(idx) = runner {
            match to {
                1 => { self.first = Some(idx); false }
                2 => { self.second = Some(idx); false }
                3 => { self.third = Some(idx); false }
                4 => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

// ── Count ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Count {
    pub balls: u8,
    pub strikes: u8,
}

impl Count {
    /// Returns true if this ball caused a walk.
    pub fn add_ball(&mut self) -> bool {
        self.balls += 1;
        self.balls >= 4
    }

    /// Returns true if this strike caused a strikeout.
    pub fn add_strike(&mut self) -> bool {
        self.strikes += 1;
        self.strikes >= 3
    }

    /// Foul: increment strikes only if fewer than 2.
    pub fn add_foul(&mut self) {
        if self.strikes < 2 {
            self.strikes += 1;
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

// ── Team ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub lineup: Vec<LineupSlot>,              // always 9 entries
    pub pitchers: Vec<PitcherAppearance>,
    pub current_pitcher_idx: usize,
    pub batting_order_pos: usize,             // 0–8, wraps mod 9
}

impl Team {
    pub fn new(name: String, lineup: Vec<LineupSlot>, starter: PitcherInfo) -> Self {
        Team {
            name,
            lineup,
            pitchers: vec![PitcherAppearance {
                info: starter,
                stats: PitcherGameStats::default(),
                entered_inning: 1,
                decision: None,
            }],
            current_pitcher_idx: 0,
            batting_order_pos: 0,
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

    pub fn advance_batter(&mut self) {
        self.batting_order_pos = (self.batting_order_pos + 1) % 9;
    }

    pub fn total_hits(&self) -> u8 {
        self.lineup.iter().map(|s| s.stats.hits).sum()
    }
}

// ── Inning score ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InningScore {
    pub away_runs: u8,
    pub home_runs: u8,
}

// ── Play log ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayLogEntry {
    pub inning: u8,
    pub half: Half,
    pub batter_name: String,
    pub description: String,
    pub rbi: u8,
}

impl PlayLogEntry {
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

#[derive(Clone, Debug)]
pub enum PitchEvent {
    Ball,
    Strike,
    Foul,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PitchOutcome {
    CountUpdated,
    WalkForced,
    StrikeoutForced,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InningOutcome {
    Continue,
    ThreeOuts,
    GameOver,
}

// ── Error tracking ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameErrors {
    pub away: u8,
    pub home: u8,
}

// ── GameState ──────────────────────────────────────────────────────────────

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

    pub fn batting_team(&self) -> &Team {
        if self.half == Half::Top { &self.away } else { &self.home }
    }

    pub fn batting_team_mut(&mut self) -> &mut Team {
        if self.half == Half::Top { &mut self.away } else { &mut self.home }
    }

    pub fn fielding_team(&self) -> &Team {
        if self.half == Half::Top { &self.home } else { &self.away }
    }

    pub fn fielding_team_mut(&mut self) -> &mut Team {
        if self.half == Half::Top { &mut self.home } else { &mut self.away }
    }

    // ── Score totals ───────────────────────────────────────────────────────

    pub fn away_total_runs(&self) -> u8 {
        self.inning_scores.iter().map(|s| s.away_runs).sum()
    }

    pub fn home_total_runs(&self) -> u8 {
        self.inning_scores.iter().map(|s| s.home_runs).sum()
    }

    pub fn away_total_hits(&self) -> u8 { self.away.total_hits() }
    pub fn home_total_hits(&self) -> u8 { self.home.total_hits() }

    // ── Pitch processing ───────────────────────────────────────────────────

    pub fn apply_pitch(&mut self, event: PitchEvent) -> PitchOutcome {
        self.fielding_team_mut().current_pitcher_mut().stats.pitch_count += 1;
        match event {
            PitchEvent::Ball => {
                if self.count.add_ball() { PitchOutcome::WalkForced } else { PitchOutcome::CountUpdated }
            }
            PitchEvent::Strike => {
                if self.count.add_strike() { PitchOutcome::StrikeoutForced } else { PitchOutcome::CountUpdated }
            }
            PitchEvent::Foul => {
                self.count.add_foul();
                PitchOutcome::CountUpdated
            }
        }
    }

    // ── At-bat resolution ──────────────────────────────────────────────────

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
            if counts_ab { batter.stats.at_bats += 1; }
            if is_hit { batter.stats.hits += 1; }
            batter.stats.rbi += rbi;
            match &result {
                AtBatResult::Walk => batter.stats.walks += 1,
                AtBatResult::HitByPitch => batter.stats.hit_by_pitch += 1,
                AtBatResult::StrikeoutSwinging | AtBatResult::StrikeoutLooking => {
                    batter.stats.strikeouts += 1;
                }
                _ => {}
            }
        }

        // Update fielding pitcher stats
        {
            let pitcher = self.fielding_team_mut().current_pitcher_mut();
            pitcher.stats.outs_recorded += outs_this_play as u16;
            if is_hit { pitcher.stats.hits_allowed += 1; }
            pitcher.stats.runs_allowed += rbi;
            if !is_error { pitcher.stats.earned_runs += rbi; }
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
            if self.game_over { InningOutcome::GameOver } else { InningOutcome::ThreeOuts }
        } else {
            InningOutcome::Continue
        }
    }

    fn advance_bases_for_result(&mut self, result: &AtBatResult, batter_idx: usize) -> (u8, Vec<usize>) {
        match result {
            AtBatResult::Single    => self.bases.advance_all(1, batter_idx),
            AtBatResult::Double    => self.bases.advance_all(2, batter_idx),
            AtBatResult::Triple    => self.bases.advance_all(3, batter_idx),
            AtBatResult::HomeRun   => self.bases.advance_all(4, batter_idx),
            AtBatResult::Walk | AtBatResult::HitByPitch => self.bases.force_advance(batter_idx),
            AtBatResult::Error(_) => {
                // Batter reaches first; existing runners don't auto-advance
                self.bases.first = Some(batter_idx);
                (0, vec![])
            }
            AtBatResult::FieldersChoice => {
                // Lead runner is retired; batter reaches first
                if self.bases.third.is_some() { self.bases.third = None; }
                else if self.bases.second.is_some() { self.bases.second = None; }
                else { self.bases.first = None; }
                self.bases.first = Some(batter_idx);
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
                // Remove the lead runner (the one thrown out first)
                if self.bases.first.is_some() { self.bases.first = None; }
                else if self.bases.second.is_some() { self.bases.second = None; }
                else if self.bases.third.is_some() { self.bases.third = None; }
                (0, vec![])
            }
            AtBatResult::Groundout(_)
            | AtBatResult::Flyout(_)
            | AtBatResult::StrikeoutSwinging
            | AtBatResult::StrikeoutLooking => (0, vec![]),
        }
    }

    fn end_half_inning(&mut self) {
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

    // ── Manual runner advance ──────────────────────────────────────────────

    /// Move a runner from `from` base to `to` base (1-3, or 4=home/scores).
    /// Returns true if the runner scored (caller should add a run to the score).
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

    // ── Decision assignment (call at game end) ─────────────────────────────

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

    fn make_team(name: &str) -> Team {
        let lineup = (1..=9)
            .map(|i| LineupSlot {
                info: BatterInfo { name: format!("Player {}", i), season_avg: 0.250 },
                stats: BatterGameStats::default(),
            })
            .collect();
        Team::new(name.to_string(), lineup, PitcherInfo { name: "Starter".into() })
    }

    fn make_game() -> GameState {
        GameState::new(make_team("Away"), make_team("Home"))
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
        let mut bases = Bases { first: Some(0), second: Some(1), third: Some(2) };
        let (runs, scorers) = bases.force_advance(3);
        assert_eq!(runs, 1);
        assert_eq!(scorers, vec![2]);
        assert_eq!(bases.first, Some(3));
        assert_eq!(bases.second, Some(0));
        assert_eq!(bases.third, Some(1));
    }

    #[test]
    fn test_hr_clears_bases() {
        let mut bases = Bases { first: Some(0), second: Some(1), third: Some(2) };
        let (runs, _) = bases.advance_all(4, 3);
        assert_eq!(runs, 4);
        assert!(bases.first.is_none());
        assert!(bases.second.is_none());
        assert!(bases.third.is_none());
    }

    #[test]
    fn test_single_advancement() {
        let mut bases = Bases { first: Some(0), second: None, third: Some(2) };
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
}
