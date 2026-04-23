# AGENTS.md

Guidance for AI coding assistants (Cursor, Claude Code, Codex, Copilot chat
agents, etc.) working inside this repository.

This file is read automatically by agentic tooling. Humans are welcome to
read it too — it doubles as a short contributor guide.

---

## 1. What this repo is

`full-count` is a single-binary Rust application: a keyboard-driven terminal UI
for scoring baseball games in real time, built on [ratatui](https://github.com/ratatui-org/ratatui)
and [crossterm](https://github.com/crossterm-rs/crossterm). All state is local;
nothing talks to the network.

The full-user-facing documentation is a static website in [`docs/`](./docs/)
that is deployed to GitHub Pages from `master`. `README.md` is deliberately
small and points at the site.

## 2. Repo layout you need to know

| Path | Purpose |
|------|---------|
| `src/main.rs` | Entry point, CLI parsing, terminal setup, event loop. |
| `src/app.rs` | `App` state: screens, input modes, setup data, undo stack. |
| `src/game.rs` | `GameState` domain model: innings, runners, stats, substitutions, rules. |
| `src/input.rs` | Per-screen key handlers — the state machine that turns keystrokes into mutations. |
| `src/ui.rs` | ratatui rendering for every screen. |
| `src/persist.rs` | JSON save/load and `~/.full-count/saves/` directory logic. |
| `src/export.rs` | Askama-driven HTML scorecard export. |
| `templates/scorecard.html` | Askama template used by `export.rs`. |
| `docs/index.html` | Marketing landing page (GitHub Pages root). |
| `docs/guide/*.html` | Docusaurus-style user guide (introduction, getting started, scoring reference, replay, saves, advanced stats, development). |
| `docs/assets/styles.css` | Shared styles for the entire site. |
| `.github/workflows/ci.yml` | Test + lint CI. |
| `.github/workflows/pages.yml` | Builds and deploys the `docs/` site to GitHub Pages on every push to `master` that touches docs. |
| `.github/copilot-instructions.md` | Parallel rules for GitHub Copilot. |

## 3. Golden rules

1. **Every user-facing change gets a docs update in the same change.**
   The documentation site under `docs/` is part of the product. If you add,
   remove, or change anything a user can see or type, update `docs/` *in the
   same commit or PR* as the code change. No "docs later".

2. **Tests pass, lints are clean.** `cargo test`, `cargo test --features advanced-stats`,
   `cargo fmt --check`, and `cargo clippy -- -D warnings` (with and without
   `--features advanced-stats`) must all pass before you consider the change
   done. CI enforces this.

3. **Keep it keyboard-first.** Every feature must be reachable without a mouse.
   If you're tempted to add something that only works with mouse events,
   stop and re-design.

4. **No network.** Do not introduce runtime network calls, telemetry, analytics,
   update checks, or implicit downloads. The app runs offline forever.

5. **Small, test-covered changes.** Prefer unit tests colocated with the code
   (`#[cfg(test)] mod tests { ... }`) over sweeping refactors.

## 4. Docs-with-code expectations

These are the specific docs-update obligations. If any item on the left
changes, update *at minimum* the pages on the right.

| Code area | Docs pages that must be re-checked |
|-----------|-------------------------------------|
| Any key binding in `src/input.rs` | `docs/guide/scoring.html` and the keygrid in `docs/index.html` |
| Pitch logic, counts, auto walks/strikeouts | `docs/guide/scoring.html` (Pitches, Auto-walk callout) |
| At-bat result handling (hits, outs, errors, FC) | `docs/guide/scoring.html` (At-bat results, Fielder-driven outs) |
| RBI prompt flow | `docs/guide/scoring.html` (RBI prompt) |
| Batter/pitcher substitutions | `docs/guide/scoring.html` (Substitutions), `docs/guide/getting-started.html` |
| Manual runner advancement | `docs/guide/scoring.html` (Manual runner advancement) |
| Undo stack depth or semantics | `docs/guide/scoring.html` (Undo), landing-page feature card |
| Replay snapshots, replay UI, `--replay` flag | `docs/guide/replay.html` |
| CLI flags in `src/main.rs` (`clap` derive) | `docs/guide/getting-started.html`, `docs/guide/saves.html` |
| Save-path resolution or name sanitization in `src/persist.rs` | `docs/guide/saves.html` (Save files, Name sanitization) |
| `~/.full-count/` directory structure | `docs/guide/saves.html` |
| HTML scorecard export in `src/export.rs` / `templates/scorecard.html` | `docs/guide/saves.html` (HTML scorecard export) |
| `advanced-stats` feature or any stat it unlocks (`Cargo.toml`) | `docs/guide/advanced-stats.html`, landing-page copy |
| New module, renamed file, changed build/test commands | `docs/guide/development.html` (Module layout, Building, Running tests) |
| **Any user-visible change whatsoever** | **Add a bullet to `<h3>Unreleased</h3>` in `docs/guide/development.html#changelog` in the same PR. See [§6 Changelog protocol](#6-changelog-protocol-mandatory).** |

When you add or rename a guide page, also update:

- The sidebar `<aside class="sidebar">` block in **every** page under `docs/guide/`.
- The "On this page" nav links in `docs/index.html` if the new page belongs in the top-level site nav.
- The pager (`<div class="pager">`) at the bottom of the previous and next pages so links stay correct.

## 5. Style conventions for the docs site

- Pure HTML + a single shared `docs/assets/styles.css`. **No build tools**, no
  JS frameworks, no Node dependency. If you catch yourself reaching for a
  bundler, stop.
- Use `<kbd>` for keys, `<code>` for filenames / identifiers / flags,
  `<pre>` for multi-line samples.
- Keep the color palette and typography consistent with the existing pages.
  Don't introduce new design tokens without updating `styles.css`.
- Mobile-friendly: don't add layouts that break the sidebar's responsive
  behavior around the `860px` breakpoint.
- All pages link to the same footer and header; keep `active` on the correct
  sidebar item.
- `.nojekyll` lives at `docs/.nojekyll` — do not delete it, or underscored
  paths may stop working on GitHub Pages.

## 6. Changelog protocol (mandatory)

Every change that affects user-visible behavior must add a bullet to the
changelog **in the same commit / PR** as the code change. This is not
optional, and it is not a follow-up task.

**Where.** The changelog lives under the `<h3>Unreleased</h3>` heading in
[`docs/guide/development.html#changelog`](./docs/guide/development.html#changelog).
There is no `CHANGELOG.md` — the site is the single source of truth. Do not
create a root-level `CHANGELOG.md`; the build assumes it doesn't exist.

**How.** Add a new `<li>` inside the `<ul>` that follows the `<h3>Unreleased</h3>`
heading. One short sentence, past tense, plain prose.

```html
<h3>Unreleased</h3>
<ul>
  <li>Added <code>B</code> as a balk reason in the manual runner-advance flow.</li>
  <!-- existing bullets… -->
</ul>
```

If there is no Unreleased section (e.g. right after a release was cut), create
a fresh one above the most recent version heading:

```html
<h3>Unreleased</h3>
<ul>
  <li>Your bullet here.</li>
</ul>

<h3>0.1.0</h3>
<!-- existing release notes… -->
```

**Release cadence.** When a release ships, rename `Unreleased` to the new
version number (e.g. `<h3>0.2.0</h3>`), bump `version` in `Cargo.toml` to
match, and leave a new empty `Unreleased` section in place for the next
change.

**What counts as user-visible.** Keybindings, CLI flags, prompts, save file
fields, directory paths, sanitization rules, HTML scorecard output, error
messages, default values, new stats, new features — all require a bullet.
Pure internal refactors with zero observable change do not. **When in doubt,
add the bullet.** A PR that changes behavior without touching the Unreleased
list is incomplete.

## 7. Commit & PR hygiene

- Conventional-ish commit subjects are fine (`scoring: add balk reason to
  runner advance`). They are not enforced.
- A good PR description mentions both the code change and the docs change.
  If an AI assistant opens a PR that touches user-facing behavior without a
  matching `docs/` diff, that PR should be treated as incomplete.
- Run `cargo fmt` and `cargo clippy --all-features -- -D warnings` locally
  before pushing to avoid CI round-trips.

## 8. Useful shell snippets

```bash
# Build and test, matching CI.
cargo fmt --check
cargo clippy -- -D warnings
cargo clippy --features advanced-stats -- -D warnings
cargo test
cargo test --features advanced-stats

# Preview the docs site locally (any static file server works).
python3 -m http.server --directory docs 8080
# then open http://localhost:8080/
```

## 9. When in doubt

If you are unsure whether a change is "user-visible" enough to warrant a docs
update, the answer is **yes, update the docs**. The cost of a paragraph of
HTML is tiny; the cost of a stale docs site is a confused user.
