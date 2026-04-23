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

### Prebuilt binaries (recommended)

1. Open [**Releases**](https://github.com/jakenherman/full-count/releases) and download the archive for your platform:
   - **Linux (x86_64):** `full-count-*-x86_64-unknown-linux-gnu.tar.gz`
   - **macOS (Apple Silicon):** `full-count-*-aarch64-apple-darwin.tar.gz`
   - **macOS (Intel):** `full-count-*-x86_64-apple-darwin.tar.gz`
   - **Windows (x86_64):** `full-count-*-x86_64-pc-windows-msvc.zip`
2. Extract the `full-count` executable (or `full-count.exe` on Windows) and put it on your `PATH`, or run it from the folder where you extracted it.

Release builds use the default feature set. For **advanced stats**, install from source with `--features advanced-stats` (see below).

### Install with Cargo (no Rust install of the binary via crates.io yet)

If you have [Rust](https://rustup.rs/) installed:

```bash
cargo install --git https://github.com/jakenherman/full-count.git --locked
# optional:
cargo install --git https://github.com/jakenherman/full-count.git --locked --features advanced-stats
```

`cargo install` compiles on your machine and places `full-count` in `~/.cargo/bin` (ensure that directory is on your `PATH`).

### Build from source

```bash
git clone https://github.com/jakenherman/full-count.git
cd full-count
cargo build --release
# Binary: target/release/full-count
# optional: cargo build --release --features advanced-stats
```

Requires Rust **1.75+** (see `rust-version` in `Cargo.toml`).

### Changelog

Version history and notable changes: [`CHANGELOG.md`](CHANGELOG.md).

### Cutting a release (maintainers)

1. Update `CHANGELOG.md`: move items from **Unreleased** into a dated section, set the version and compare links at the bottom.
2. Set `version` in `Cargo.toml` to match the release (for example `0.2.0`).
3. Commit, then tag and push:

   ```bash
   git tag -a v0.2.0 -m "v0.2.0"
   git push
   git push --tags
   ```

4. The [**Release**](.github/workflows/release.yml) workflow builds Linux, Windows, and macOS artifacts and attaches them to a **[GitHub Release](https://github.com/jakenherman/full-count/releases)** for that tag. Edit the release notes if you want more detail than the auto-generated summary.

Optional later steps: publish to [crates.io](https://crates.io/) with `cargo publish` so users can run `cargo install full-count` (after claiming the crate name and adding any metadata crates.io requires).

## Usage

```bash
full-count                              # new game
full-count --load cubs-vs-sox           # resume scoring
full-count --load cubs-vs-sox --replay  # step through pitch-by-pitch
```

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

## Astros Scoreability Pull Script

Use `scripts/pull_astros_scoreability.py` to pull Astros MLB game data and check whether all plays are representable by Full Count.

```bash
# Pull by date (Astros schedule on that date)
python3 scripts/pull_astros_scoreability.py --date 2026-04-09

# Pull a specific game directly
python3 scripts/pull_astros_scoreability.py --date 2026-04-09 --game-pk 824374
```

By default, output is written under `test_data/mlb/`:

- Fixture JSON: `astros-scoreability-<date>.json` (or `...-game-<gamePk>.json`)
- Issue drafts: `test_data/mlb/issues/<query-date>/...` for unsupported event types
- These files are generated artifacts and are gitignored.

Helpful flags:

- `--date YYYY-MM-DD` query date (defaults to today)
- `--game-pk <id>` target one specific game
- `--out-dir <path>` write fixtures/issues somewhere else

## Development

```bash
cargo test                              # run the test suite
cargo test --features advanced-stats
cargo build --release                   # release build
cargo fmt --check
cargo clippy -- -D warnings
```

See [`AGENTS.md`](./AGENTS.md) for contributor rules — especially the
documentation-with-code expectations if you're using an AI assistant.

## License

[MIT](./LICENSE)
