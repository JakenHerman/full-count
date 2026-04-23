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
| **Any user-visible change whatsoever** | **Write a [Conventional Commit](https://www.conventionalcommits.org/) subject (`feat:`, `fix:`, `feat!:`, …). See [§6 Changelog & release protocol](#6-changelog--release-protocol-mandatory).** |

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

## 6. Changelog & release protocol (mandatory)

`CHANGELOG.md` and version bumps are **fully automated** by
[`release-plz`](https://release-plz.ieni.dev/) (see `release-plz.toml` and
`.github/workflows/release-plz.yml`). Agents do not hand-edit any of:

- `CHANGELOG.md`
- `version` in `Cargo.toml`
- `Cargo.lock` version rows
- The changelog section of `docs/guide/development.html`

Those files are regenerated from commit messages when release-plz opens its
`chore: release` PR. Touching them in a feature PR will cause merge conflicts
against the Release PR — don't.

### The one rule: use Conventional Commits

Every commit that lands on `master` (either directly or via squash-merge,
whichever the PR uses) **must** use a
[Conventional Commit](https://www.conventionalcommits.org/) subject. This is
how release-plz knows a release is needed and what kind of version bump to
cut.

| Subject prefix | Meaning | Triggers a release? | Bump |
|----------------|---------|---------------------|------|
| `feat: …` | New user-visible feature or behavior | Yes | MINOR (e.g. 0.1.0 → 0.2.0) |
| `fix: …` | Bug fix | Yes | PATCH (e.g. 0.2.0 → 0.2.1) |
| `feat!: …` or any `BREAKING CHANGE:` footer | Breaking user-visible change | Yes | MINOR while pre-1.0, MAJOR after |
| `docs: …` | Docs site or comments only | No | — |
| `chore: …`, `refactor: …`, `test: …`, `ci: …`, `style: …` | Internal, no user impact | No | — |
| `perf: …` | Performance improvement users would notice | Yes | PATCH |

Scopes are optional but encouraged for clarity:
`feat(scoring): add balk reason to runner advance`.

Examples that are good:

- `feat(scoring): add B as a balk reason in manual runner advance`
- `fix(persist): sanitize colons in save filenames on Windows`
- `feat!(saves): add pitch_id field to save schema`
- `docs(scoring): document balk reason`
- `chore: bump ratatui to 0.30`

### What agents must do

1. **Pick the right prefix.** If the change is user-visible, it must be
   `feat`, `fix`, `feat!`, or `perf` — never `chore` or `refactor`. When in
   doubt, prefer `feat` or `fix` over `chore`; a missing bullet is worse than
   an extra one.
2. **Write the subject like a changelog entry.** It will appear verbatim in
   `CHANGELOG.md`. One line, imperative mood, no trailing period, no ticket
   IDs unless they add real context.
3. **Still update `docs/` in the same PR** for any user-visible change (see
   §4). The commit-message bullet is the changelog; the docs site is the
   reference manual. They are separate obligations.
4. **Do not touch `CHANGELOG.md`, `release-plz.toml`, or the version in
   `Cargo.toml`** except in a deliberate repo-maintenance PR.

### What counts as user-visible

Keybindings, CLI flags, prompts, save file fields, directory paths,
sanitization rules, HTML scorecard output, error messages, default values,
new stats, new features — all require `feat`/`fix`/`feat!`. Pure internal
refactors with zero observable change get `refactor` or `chore`. **When in
doubt, use `feat` or `fix`.**

### How releases actually ship

This is informational — agents don't run any of these steps themselves.

1. Conventional commits land on `master`.
2. The `release-plz` workflow opens (or updates) a `chore: release` PR that
   bumps `Cargo.toml`, refreshes `Cargo.lock`, and appends an entry to
   `CHANGELOG.md`.
3. A maintainer reviews and merges the Release PR.
4. `release-plz` pushes the `vX.Y.Z` tag and creates the GitHub Release.
5. The tag push triggers `release.yml`, which builds Linux / macOS / Windows
   binaries and attaches them to the release.

## 7. Commit & PR hygiene

- **Commit subjects must be [Conventional Commits](https://www.conventionalcommits.org/).**
  See §6 for the allowed prefixes and what each one does to the version. The
  subject is the changelog entry — write it as if a user will read it,
  because they will.
- A good PR description mentions both the code change and the docs change.
  If an AI assistant opens a PR that touches user-facing behavior without a
  matching `docs/` diff, that PR should be treated as incomplete.
- Run `cargo fmt` and `cargo clippy --all-features -- -D warnings` locally
  before pushing to avoid CI round-trips.
- Do not bump `version` in `Cargo.toml`, touch `Cargo.lock`'s version rows,
  or edit `CHANGELOG.md` as part of a feature PR — `release-plz` owns those
  files (§6).

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
