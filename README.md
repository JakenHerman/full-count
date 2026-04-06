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
- **Pitcher changes** — unlimited relievers with W/L/S decisions at game end
- **Manual runner advancement** — wild pitches, stolen bases, passed balls, balks
- **Save & resume** — JSON saves at `~/.full-count/saves/`
- **Game replay** — step through any saved game pitch-by-pitch in read-only mode
- **Undo** — up to 100 levels
- **Advanced stats** — compile with `--features advanced-stats` for 2B/3B/HR, SB/CS, LOB, WP, BF, season AVG

## Install

```bash
cargo build --release
# optionally: cargo build --release --features advanced-stats
```

Requires [Rust](https://rustup.rs/) 1.75+.

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

**Other:** `A` advance runner · `Tab` pitcher change · `U` undo · `F2` save · `X` end game · `Q` quit

After hits/walks/FC, an RBI prompt appears — type `0`–`4` and `Enter`. Press `Esc` to cancel any prompt.

## Replay Mode

Load a saved game in replay mode from the load menu (`R` instead of `Enter`) or via CLI (`--replay`).

`←`/`H` step back · `→`/`L` step forward · `g` jump to start · `G` jump to end · `Esc` exit

Replay data is captured automatically during scoring and persisted with each save.

## Save Files

Games save to `~/.full-count/saves/<name>.json`. Names are sanitized (spaces → hyphens, max 64 chars). Save with `F2` during a game, resume with `--load`.

## Development

```bash
cargo test              # 137 tests
cargo build --release   # release build
```

## License

MIT
