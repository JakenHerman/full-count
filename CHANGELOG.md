# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-10

### Added

- Keyboard-driven TUI for live baseball scoring ([ratatui](https://github.com/ratatui-org/ratatui))
- Pitch-by-pitch tracking (balls, strikes, fouls) with walk and strikeout handling
- At-bat results: hits, outs, walks, HBP, errors, double plays, sac flies, fielder's choice
- Standard fielder notation (for example `6-4-3`, `F8`, `E6`)
- Live scoreboard with inning line score and R/H/E
- Batter and pitcher stat lines updated as you score
- Mid-game batter and pitcher changes with retained lines
- Manual runner advancement (wild pitch, stolen base, passed ball, balk, etc.)
- JSON save and resume under `~/.full-count/saves/`
- Replay mode: step through saved games with keyboard navigation
- Undo (up to 100 steps)
- Optional `advanced-stats` feature: extra counting stats (2B/3B/HR, SB/CS, LOB, WP, BF, season AVG, and related fields)
- HTML scorecard export (Askama templates)
- Pre-built release binaries for Linux (x86_64), macOS (Apple Silicon and Intel), and Windows (x86_64) via GitHub Releases

[Unreleased]: https://github.com/jakenherman/full-count/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jakenherman/full-count/releases/tag/v0.1.0
