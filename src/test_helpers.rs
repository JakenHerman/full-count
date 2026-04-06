/// Shared test helpers — compiled only in test builds.
#[cfg(test)]
pub mod helpers {
    use crate::game::{BatterGameStats, BatterInfo, GameState, LineupSlot, PitcherInfo, Team};

    pub fn make_team(name: &str) -> Team {
        let lineup = (1..=9)
            .map(|i| LineupSlot {
                info: BatterInfo { name: format!("Player {}", i), season_avg: 0.250 },
                stats: BatterGameStats::default(),
            })
            .collect();
        Team::new(name.to_string(), lineup, PitcherInfo { name: "Starter".into() })
    }

    pub fn make_game() -> GameState {
        GameState::new(make_team("Away"), make_team("Home"))
    }
}
