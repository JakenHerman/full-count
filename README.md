# full-count

A keyboard-driven terminal UI (TUI) for scoring baseball games in real time. Built in Rust with [ratatui](https://github.com/ratatui-org/ratatui).

```
  _____ _   _ _     _       ____ ___  _   _ _   _ _____
 |  ___| | | | |   | |     / ___/ _ \| | | | \ | |_   _|
 | |_  | | | | |   | |    | |  | | | | | | |  \| | | |
 |  _| | |_| | |___| |___ | |__| |_| | |_| | |\  | | |
 |_|    \___/|_____|_____| \____\___/ \___/|_| \_| |_|

         ⚾ Baseball Scoring TUI ⚾
```

---

## Features

- **Pitch-by-pitch tracking** — balls, strikes, fouls; automatic walk/strikeout detection
- **Full at-bat result vocabulary** — hits, outs, walks, errors, double plays, sacrifice flies, fielder's choice, and more
- **Standard fielder-sequence notation** — enter plays like `6-4-3` (SS to 2B to 1B double play)
- **Live scoreboard** — inning-by-inning line score with R/H/E totals
- **Batter and pitcher stat lines** — AB/R/H/RBI/BB/K/HBP and IP/H/R/ER/BB/K/HBP/PC tracked automatically
- **Pitcher changes** — unlimited relievers, each with their own stat line and W/L/S decision
- **Manual runner advancement** — move any runner to any base or home
- **Play log** — scrollable chronological record of every at-bat
- **Save & resume** — games are serialized to JSON at `~/.full-count/saves/`; resume mid-inning
- **Undo** — up to 100 levels of undo across pitch events and at-bat results
- **Walk-off detection** — game ends automatically when the home team takes the lead in the 9th or later
- **Optional season batting averages** — compile with `--features season-avg` to enter and display pre-game AVGs

---

## Installation

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, 1.75+)

### Build

```bash
git clone https://github.com/your-username/full-count
cd full-count
cargo build --release
```

The binary lands at `target/release/full-count`. Copy it anywhere on your `$PATH`.

### With season batting average support

```bash
cargo build --release --features season-avg
```

This adds an **Avg** field to the lineup setup form and displays each batter's season average in the scoring view.

---

## Quick Start

```bash
# Start a new game interactively
full-count

# Resume a previously saved game
full-count --load cubs-vs-sox

# Full path also works
full-count --load ~/my-saves/game1.json
```

The `--load` flag accepts:
- A bare name (`game1`) — looked up in `~/.full-count/saves/`
- A name with extension (`game1.json`) — same lookup
- A relative or absolute path to any `.json` save file

---

## Screens

### Title Screen

```
  ❯  New Game
     Load Game
     Quit
```

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor |
| `Enter` | Select |

---

### Setup Screen

Enter team names, the 9-slot batting lineup, and starting pitchers. Tab through every field.

| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Enter` (on last field) | Start the game |
| `F3` | Open Load Game menu |
| `Backspace` | Delete character |

**Lineup slots** default to `Away #1` … `Away #9` if left blank, so you can jump straight to pitchers and start scoring without entering names.

When the `season-avg` feature is enabled, each lineup row gains an **Avg** column that accepts values like `325` or `.325` (both parsed identically as `.325`).

---

### Scoring Screen

The main screen. All scoring is driven by single keypresses.

#### Pitch Events

| Key | Event | Rule reference |
|-----|-------|----------------|
| `B` | Ball | OBR Rule 5.04(b)(1) — four balls = walk |
| `S` | Strike (swinging) | OBR Rule 5.04(b)(2) — three strikes = strikeout |
| `F` | Foul ball | OBR Rule 5.05(b)(5) — foul cannot be third strike unless bunted |

When `B` produces the 4th ball the game automatically transitions to a walk (RBI prompt opens). When `S` produces the 3rd strike a swinging strikeout is recorded immediately.

#### At-Bat Results (direct — no pitch count required)

| Key | Result | Scoring abbreviation |
|-----|--------|----------------------|
| `1` | Single | `1B` |
| `2` | Double | `2B` |
| `3` | Triple | `3B` |
| `H` | Home run | `HR` |
| `K` | Strikeout swinging | `K` |
| `L` | Strikeout looking | `Kl` |
| `W` | Walk (base on balls) | `BB` |
| `P` | Hit by pitch | `HBP` |
| `C` | Fielder's choice | `FC` |
| `G` | Groundout → fielder prompt | e.g. `6-3` |
| `D` | Double play → fielder prompt | e.g. `6-4-3 DP` |
| `O` | Flyout → fielder prompt | e.g. `F8` |
| `E` | Error → fielder prompt | e.g. `E6` |
| `V` | Sacrifice fly → fielder prompt | e.g. `SF9` |

After any result that can have RBIs (hits, walks with bases loaded, fielder's choice, etc.) a prompt asks `RBI: _`. Type `0`–`4` and press `Enter`.

#### Fielder-Sequence Prompts

Several results ask you to enter the fielder(s) involved. Use standard baseball fielder position numbers separated by `-`:

| Position | Fielder |
|----------|---------|
| 1 | Pitcher (P) |
| 2 | Catcher (C) |
| 3 | First Baseman (1B) |
| 4 | Second Baseman (2B) |
| 5 | Third Baseman (3B) |
| 6 | Shortstop (SS) |
| 7 | Left Fielder (LF) |
| 8 | Center Fielder (CF) |
| 9 | Right Fielder (RF) |

> Position numbering is defined in the **Official Baseball Rules (OBR) Rule 2.00 — Definitions of Terms**. The numbering has been standard since at least the 1950s and is used on all official scoresheets.

**Examples:**

```
G  →  6-3       Shortstop to first (routine groundout)
G  →  5-4-3     Third to second to first (around-the-horn)
D  →  6-4-3     Classic 6-4-3 double play
D  →  4-3       Second baseman unassisted to first
O  →  8         Center fielder flyout
E  →  6         Shortstop error
V  →  9         Right fielder sacrifice fly
```

Press `Esc` at any prompt to cancel and return to waiting.

#### Other Scoring Actions

| Key | Action |
|-----|--------|
| `Tab` | Pitcher change — prompts for new pitcher's name |
| `A` | Manually advance a runner (prompts for from/to base) |
| `U` | Undo last action (up to 100 levels) |
| `↑` / `↓` | Scroll the play log |
| `F2` | Save game — prompts for a save name |
| `F3` | Open Load Game menu |
| `X` | End game and view the summary |
| `Q` | Quit without saving |

#### Runner Advancement (`A`)

After pressing `A`:

1. The status bar shows `Move runner from base: [1] [2] [3]  [Esc] cancel`
2. Press the **source base** (`1`, `2`, or `3`)
3. The status bar shows `To base: [1] [2] [3] [H]ome  [Esc] cancel`
4. Press the **destination base** (`1`–`3`) or `H` to score the runner

This handles situations the automatic base-advancement logic doesn't cover: wild pitch advances, passed balls, balks, stolen bases, caught-stealing outs (remove with undo), and unusual plays.

> **OBR Rule 9.12** governs how scorers credit baserunner advancement. Advances on wild pitches (WP), passed balls (PB), balks, and stolen bases (SB) are not differentiated in the current TUI — use the play log and your own notes for those distinctions. See [Wiki: Tracking Wild Pitches and Stolen Bases](#wiki-pages) for recommended conventions.

#### Pitcher Changes (`Tab`)

Pressing `Tab` opens a prompt for the relieving pitcher's name. The new pitcher's stats start fresh. W/L/S decisions are assigned automatically at game end using the following logic:

- **Win**: the pitcher of record when the winning team took the lead for good (simplified: penultimate pitcher on the winning team if multiple pitchers, starter if only one)
- **Loss**: last pitcher on the losing team
- **Save**: last pitcher on the winning team (when there are 2+ pitchers)

> Official save rules are defined in **OBR Rule 9.19**. The current logic applies a simplified version suitable for informal scoring; for games where the official save rule matters (e.g. league stats), see [Wiki: Save Rule Details](#wiki-pages).

---

### Load Game Screen

```
  Cubs vs White Sox  |  Top 7  |  3-2
  Cardinals vs Reds  |  Bot 4  |  0-0
```

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor |
| `Enter` | Load highlighted game |
| `Esc` | Return to title |

Saves are listed newest-first. Each slot shows team names, inning/half, and the current score.

---

### Summary Screen

Displays the final box score, batter stat lines, and the pitching staff with W/L/S decisions. Press `Q` or `Esc` to exit.

---

## Stat Tracking Reference

### Batter Stats

| Column | Meaning | OBR Reference |
|--------|---------|---------------|
| AB | At-bats | Rule 9.02(a) — excludes BB, HBP, SF |
| R | Runs scored | Rule 9.04 |
| H | Hits | Rule 9.05 |
| RBI | Runs batted in | Rule 9.04 |
| BB | Bases on balls (walks) | Rule 9.08 |
| K | Strikeouts | Rule 9.15 |
| HBP | Hit by pitch | Rule 9.01(c) |

**Batting average** (when `season-avg` feature is enabled) is a pre-game entry and is not recalculated from game stats.

### Pitcher Stats

| Column | Meaning | OBR Reference |
|--------|---------|---------------|
| IP | Innings pitched (e.g. `6.2`) | Rule 9.17 |
| H | Hits allowed | Rule 9.05 |
| R | Runs allowed | Rule 9.04 |
| ER | Earned runs | Rule 9.16 |
| BB | Walks issued | Rule 9.08 |
| K | Strikeouts | Rule 9.15 |
| HBP | Hit batsmen | — |
| PC | Pitch count | — |

**Innings pitched notation**: `6.2` means 6 full innings plus 2 additional outs (i.e. 20 outs recorded). The `.1`, `.2` suffixes represent outs, not fractions of an inning. A pitcher who gets one out in the 7th before being relieved is credited with `0.1` IP for that appearance.

> **OBR Rule 9.17** defines innings-pitched calculation. Fractions are always expressed in thirds (`.1` = ⅓ inning, `.2` = ⅔ inning).

### Earned vs. Unearned Runs

When a result is scored as `Error (E)`, subsequent runs in that inning that would not have scored without the error are unearned. The current TUI credits runs on an error at-bat as **unearned** (not added to ER) only for runs scored *on the error play itself*. Complex unearned-run determinations across multiple batters require manual judgment. See [Wiki: Earned Run Calculation](#wiki-pages).

---

## Scoring Examples

### Example 1: Routine inning

```
Top 1st, no outs, count 0-0

B         → 1-0
S         → 1-1
G [6-3]   → Shortstop to first, 1 out

S         → 0-1
S         → 0-2
F         → 0-2 (foul, no change)
K         → Strikeout swinging, 2 outs

1         → Single to left, RBI: 0
  Runner on first
H         → Home run, RBI: 2
  2-0 Away after top 1st
```

### Example 2: Double play

```
Bot 3rd, 0 outs, runner on first

G         → press G, fielder prompt appears
  Enter: 6-4-3
  → 6-4-3 DP recorded, 2 outs, runner cleared
```

### Example 3: Bases-loaded walk

```
Top 5th, bases loaded

B         → 3-0
B         → 3-1 (typo — intended Ball)
U         → Undo last pitch
B         → back to 3-0... continue
B         → 3-1
B         → 3-2
B         → Walk! RBI prompt opens automatically
  RBI: 1  → force-advanced run scores, pitcher charged
```

### Example 4: Sacrifice fly

```
Bot 6th, runner on third, 1 out

V         → Sac fly, fielder prompt
  Enter: 9
  RBI: 1
  → SF9 recorded, run scores, 2 outs (sac fly is not an AB)
```

### Example 5: Mid-game pitcher change

```
Tab       → Pitcher change prompt
  Enter name: Williams
  → "Now pitching: Williams" in status bar
  → All subsequent pitching stats credited to Williams
```

### Example 6: Wild pitch advance (manual)

```
Runner on first, pitcher uncorks a wild pitch

A         → Runner advance mode
  From: 1
  To:   2
  → Runner moved to second (no automatic stat credited)
```

---

## Save Files

Games are saved to `~/.full-count/saves/<name>.json`. The format is a JSON object wrapping the full game state:

```json
{
  "saved_at_secs": 1712345678,
  "game": {
    "inning": 5,
    "half": "Top",
    "outs": 1,
    ...
  }
}
```

Save names are sanitized: spaces and special characters become `-`, the name is capped at 64 characters, and `.json` is appended automatically.

```bash
# Save named "cubs-vs-sox-game1"
F2 → cubs vs sox game1   (spaces become hyphens)
→ saved as cubs-vs-sox-game1.json
```

**Resuming from the command line:**

```bash
full-count --load cubs-vs-sox-game1
full-count --load cubs-vs-sox-game1.json    # same
full-count --load ~/.full-count/saves/cubs-vs-sox-game1.json   # absolute path
```

---

## Official Rules References

The scoring conventions implemented in `full-count` follow the **Official Baseball Rules (OBR)** published annually by Major League Baseball. Key sections:

| Topic | OBR Rule |
|-------|----------|
| Definitions (fielder positions, fair/foul, etc.) | Rule 2.00 |
| Ball, strike, foul definitions | Rules 2.00, 5.05 |
| Batting out of turn | Rule 6.03(b) *(not enforced by TUI)* |
| Base on balls | Rule 5.05(b)(1), Rule 9.08 |
| Hit by pitch | Rule 5.05(b)(2) |
| At-bat vs. plate appearance | Rule 9.02 |
| Runs and RBI | Rule 9.04 |
| Hits | Rule 9.05 |
| Errors | Rule 9.12 |
| Sacrifice fly | Rule 9.02(a)(3) |
| Strikeouts | Rule 9.15 |
| Walks | Rule 9.08 |
| Innings pitched | Rule 9.17 |
| Earned runs | Rule 9.16 |
| Pitcher win/loss/save | Rules 9.17, 9.19 |
| Double play | Rule 2.00 |

The full rulebook is available at [mlb.com/official-information/umpires/official-rules](https://www.mlb.com/official-information/umpires/official-rules) and is published each spring. For historical context and amateur/youth rule variations see the relevant governing body's rulebook (USA Baseball, Little League, NCAA, etc.).

A useful independent reference is the **Baseball Scorecard Guide** by Retrosheet, which documents traditional scorekeeping symbols used by official scorers:
[retrosheet.org/scorekeeping.htm](https://www.retrosheet.org/scorekeeping.htm)

---

## Wiki Pages

The following topics are detailed enough to warrant their own wiki pages. The README will link to them once they exist.

- **Wiki: Tracking Wild Pitches and Stolen Bases** — conventions for using runner advance (`A`) to log WP, PB, SB, CS, and balks; how to note them in the play log
- **Wiki: Earned Run Calculation** — working through multi-error innings and determining earned vs. unearned runs per OBR Rule 9.16
- **Wiki: Save Rule Details** — full OBR Rule 9.19 save criteria vs. the simplified logic the TUI uses; how to manually record saves that the TUI assigns incorrectly
- **Wiki: Defensive Indifference and Other Edge Cases** — fielder's choice variants, interference, obstruction, infield fly rule, dropped third strike
- **Wiki: Keeping a Traditional Scorecard Alongside the TUI** — suggested paper scorecard symbols to supplement what the TUI captures
- **Wiki: Exporting Game Data** — parsing the JSON save files to produce box scores, season stat summaries, or import into other tools

---

## Development

```bash
# Run tests
cargo test

# Run with season-avg feature
cargo run --features season-avg

# Release build
cargo build --release
```

The codebase is organized as:

| File | Role |
|------|------|
| `src/main.rs` | CLI parsing, terminal setup, event loop |
| `src/game.rs` | Game state, rules engine, unit tests |
| `src/app.rs` | Application state, screen/input modes, undo, save/load wiring |
| `src/input.rs` | Keyboard event handlers for each screen |
| `src/ui.rs` | Ratatui rendering for all screens |
| `src/persist.rs` | JSON serialization, `~/.full-count/saves/` management |

---

## License

MIT
