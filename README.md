<p align="center">
  <img src="baseball-comic.jpeg" alt="Sure, baseball is boring, but if you learn how to keep score it's also math." width="500">
</p>

# full-count

A keyboard-driven TUI for scoring baseball games in real time. Built in Rust with [ratatui](https://github.com/ratatui-org/ratatui).

```
  _____ _   _ _     _       ____ ___  _   _ _   _ _____
 |  ___| | | | |   | |     / ___/ _ \| | | | \ | |_   _|
 | |_  | | | | |   | |    | |  | | | | | | |  \| | | |
 |  _| | |_| | |___| |___ | |__| |_| | |_| | |\  | | |
 |_|    \___/|_____|_____| \____\___/ \___/|_| \_| |_|

         ⚾ Every pitch. Every play. Every out. ⚾
```

## Documentation

Full docs — install guide, scoring reference, replay mode, saves/exports, and
contributor notes — live at **[the documentation site](https://jakenherman.github.io/full-count/)**,
or in [`docs/`](./docs/) if you prefer to browse them locally.

- **[Introduction](https://jakenherman.github.io/full-count/guide/)** — what full-count is and how it works.
- **[Why full-count?](https://jakenherman.github.io/full-count/guide/why.html)** — the case for a keyboard-first TUI.
- **[Install & first game](https://jakenherman.github.io/full-count/guide/getting-started.html)** — score a half inning in five minutes.
- **[Scoring reference](https://jakenherman.github.io/full-count/guide/scoring.html)** — every key, every prompt.
- **[Replay mode](https://jakenherman.github.io/full-count/guide/replay.html)** — step through saved games pitch-by-pitch.
- **[Saves & exports](https://jakenherman.github.io/full-count/guide/saves.html)** — files, paths, and the HTML scorecard.
- **[Advanced stats](https://jakenherman.github.io/full-count/guide/advanced-stats.html)** — the opt-in Cargo feature.
- **[Development](https://jakenherman.github.io/full-count/guide/development.html)** — building, testing, contributing.

## Quick start

```bash
cargo build --release
# optionally: cargo build --release --features advanced-stats
full-count                              # new game
full-count --load cubs-vs-sox           # resume scoring
full-count --load cubs-vs-sox --replay  # step through pitch-by-pitch
```

Requires [Rust](https://rustup.rs/) 1.75+.

## Feature highlights

- Pitch-by-pitch tracking with automatic walks and strikeouts
- Hits, outs, walks, errors, double plays, sac flies, fielder's choice
- Standard fielder notation (`6-4-3`, `F8`, `E6`)
- Live R/H/E line score, batter & pitcher stat lines
- Mid-game batter and pitcher substitutions
- Manual runner advancement (SB, CS, WP, PB, balk)
- JSON saves under `~/.full-count/saves/` with full replay snapshots
- Pitch-by-pitch replay mode and up to 100 levels of undo
- Paper-style HTML scorecard export
- Opt-in `advanced-stats` Cargo feature (2B/3B/HR, SB/CS, LOB, WP, BF, season AVG)

## Development

```bash
cargo test                              # 137 tests
cargo test --features advanced-stats
cargo fmt --check
cargo clippy -- -D warnings
```

See [`AGENTS.md`](./AGENTS.md) for contributor rules — especially the
documentation-with-code expectations if you're using an AI assistant.

## License

[MIT](./LICENSE)
