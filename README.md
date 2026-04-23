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

## Features

- **Pitch-by-pitch tracking** — balls, strikes, fouls with automatic walk/strikeout detection
- **Full at-bat vocabulary** — hits, outs, walks, errors, double plays, sac flies, fielder's choice
- **Standard fielder notation** — `6-4-3` double play, `F8` flyout, `E6` error
- **Live scoreboard** — inning-by-inning line score with R/H/E
- **Batter & pitcher stat lines** — updated automatically after every play
- **Batter changes** — swap in a new hitter mid-game without losing the replaced batter's line
- **Pitcher changes** — unlimited relievers with W/L/S decisions at game end
- **Manual runner advancement** — wild pitches, stolen bases, passed balls, balks
- **Save & resume** — JSON saves at `~/.full-count/saves/`
- **Game replay** — step through any saved game pitch-by-pitch in read-only mode
- **Undo** — up to 100 levels
- **Advanced stats** — compile with `--features advanced-stats` for 2B/3B/HR, SB/CS, LOB, WP, BF, season AVG

## Install

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

## Quick Start

```bash
full-count                              # new game
full-count --load cubs-vs-sox           # resume scoring
full-count --load cubs-vs-sox --replay  # step through pitch-by-pitch
```

`--load` accepts a bare name, `name.json`, or a full path. Bare names are looked up in `~/.full-count/saves/`.

## Scoring Keys

**Pitches:** `B` ball · `S` strike · `F` foul

**At-bat results:** `1` single · `2` double · `3` triple · `H` homer · `K` K-swing · `L` K-look · `W` walk · `P` HBP · `C` FC

**Fielder prompts:** `G` groundout · `D` double play · `O` flyout · `E` error · `V` sac fly — enter positions like `6-3` or `6-4-3`

**Other:** `A` advance runner · `R` batter change · `Tab` pitcher change · `U` undo · `F2` save · `X` end game · `Q` quit

After hits/walks/FC, an RBI prompt appears — type `0`–`4` and `Enter`. Press `Esc` to cancel any prompt.

## Replay Mode

Load a saved game in replay mode from the load menu (`R` instead of `Enter`) or via CLI (`--replay`).

`←`/`H` step back · `→`/`L` step forward · `g` jump to start · `G` jump to end · `Esc` exit

Replay data is captured automatically during scoring and persisted with each save.

## Save Files

Games save to `~/.full-count/saves/<name>.json`. Names are sanitized (spaces → hyphens, max 64 chars). Save with `F2` during a game, resume with `--load`.

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
cargo test              # run the test suite
cargo build --release   # release build
```

## License

MIT
