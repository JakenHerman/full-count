# GitHub Copilot instructions

Instructions for GitHub Copilot (and Copilot Chat / Copilot coding agents)
working in this repository. Copilot picks this file up automatically when it
generates code or opens pull requests.

These rules mirror and reinforce [`AGENTS.md`](../AGENTS.md). If anything in
this file conflicts with `AGENTS.md`, `AGENTS.md` wins — update both so they
stay aligned.

---

## Project summary

`full-count` is a keyboard-driven Rust TUI for scoring baseball games in real
time, built on ratatui. It has:

- A single binary (`src/main.rs`).
- Domain logic in `src/game.rs` and `src/app.rs`.
- Keyboard input handling in `src/input.rs`.
- ratatui rendering in `src/ui.rs`.
- JSON persistence in `src/persist.rs`, HTML scorecard export in `src/export.rs`.
- An opt-in `advanced-stats` Cargo feature for extra tracked metrics.

User-facing documentation is a static website under [`docs/`](../docs/) that
is published to GitHub Pages. Copilot **must** keep that site in sync with
code changes.

## Core rules

1. **Always update the docs site in the same change as the code.**
   If a change is visible to users — new key, new flag, new save behavior, new
   stat, renamed module — update the relevant files under `docs/` in the same
   commit / PR. No "follow-up docs PR".

2. **Always use a [Conventional Commit](https://www.conventionalcommits.org/) subject.**
   The commit message is the changelog — `release-plz` generates
   `CHANGELOG.md` and bumps versions from it. See the dedicated
   [Commit & changelog rule (mandatory)](#commit--changelog-rule-mandatory)
   section below. This is not optional.

3. **Never break the no-build-tools constraint on the docs site.**
   `docs/` is pure HTML + one CSS file. Do not add a package.json, Docusaurus,
   Next.js, Astro, or any framework. Do not add JavaScript unless strictly
   necessary for an accessibility affordance, and even then prefer CSS.

4. **Keep the CSS and design tokens consistent.** Reuse the classes and
   variables already in `docs/assets/styles.css`. New tokens go in `:root`.

5. **Do not introduce network calls at runtime.** No telemetry, analytics,
   update checks, or external APIs. The app runs offline.

6. **Do not weaken CI.** `cargo test`, `cargo test --features advanced-stats`,
   `cargo fmt --check`, and `cargo clippy -- -D warnings` (with and without
   the feature) must all pass.

## Commit & changelog rule (mandatory)

`CHANGELOG.md` and `version` in `Cargo.toml` are maintained by
[`release-plz`](https://release-plz.ieni.dev/) (config in
`release-plz.toml`, workflow in `.github/workflows/release-plz.yml`). Copilot
must **never** hand-edit:

- `CHANGELOG.md`
- `version = "…"` in `Cargo.toml`
- The version entry in `Cargo.lock`
- The changelog section of `docs/guide/development.html`

Those files are regenerated whenever release-plz opens its `chore: release`
PR. Touching them in a feature PR creates merge conflicts against the Release
PR.

### The rule: Conventional Commit subjects

Every commit landing on `master` — directly or via squash-merge — must use a
[Conventional Commit](https://www.conventionalcommits.org/) subject. The
subject is the changelog entry.

| Prefix | Meaning | Releases? | Bump |
|--------|---------|-----------|------|
| `feat: …` | New user-visible behavior | Yes | MINOR |
| `fix: …` | Bug fix | Yes | PATCH |
| `feat!: …` / `BREAKING CHANGE:` footer | Breaking change | Yes | MINOR pre-1.0, MAJOR after |
| `perf: …` | User-noticeable perf improvement | Yes | PATCH |
| `docs: …` | Docs / comments only | No | — |
| `chore: …`, `refactor: …`, `test: …`, `ci: …`, `style: …` | Internal only | No | — |

**If the change is visible to a user** (new keybinding, CLI flag, prompt,
save-file field, `advanced-stats` metric, default value, error message, HTML
scorecard output, etc.), the prefix **must** be `feat`, `fix`, `feat!`, or
`perf`. `chore` and `refactor` are for changes the user could never notice.

### Good commit subjects

```
feat(scoring): add B as a balk reason in manual runner advance
fix(persist): sanitize colons in save filenames on Windows
feat!(saves): add pitch_id field to save schema
perf(ui): cache rendered scoreboard between frames
docs(scoring): document balk reason
chore: bump ratatui to 0.30
```

Rules for the subject itself:

- One line, imperative mood, no trailing period.
- No emoji, no ticket numbers unless they add real context.
- Write it as the sentence a user will read in the release notes, because
  they will.

### Still update `docs/` in the same PR

The docs site is the **reference manual**; the commit message is the
**changelog entry**. Both are required for user-visible changes — see the
[Docs-with-code mapping](#docs-with-code-mapping) below. A PR that changes
behavior without touching `docs/` is incomplete.

## Docs-with-code mapping

Every row below also requires a Conventional Commit subject
(`feat:` / `fix:` / `feat!:` / `perf:`) — see
[Commit & changelog rule](#commit--changelog-rule-mandatory). That column is
implicit and non-negotiable.

| If you touch… | Then also update at minimum… | Conventional commit? |
|---------------|------------------------------|----------------------|
| `src/input.rs` (keybindings) | `docs/guide/scoring.html`, landing-page keygrid in `docs/index.html` | **Required** |
| `src/main.rs` (CLI flags) | `docs/guide/getting-started.html`, `docs/guide/saves.html` | **Required** |
| `src/persist.rs` (paths, filenames, sanitization) | `docs/guide/saves.html` | **Required** |
| `src/export.rs` or `templates/scorecard.html` | `docs/guide/saves.html` (HTML scorecard export) | **Required** |
| `src/game.rs` rules (outs, RBI, advancement, substitutions) | `docs/guide/scoring.html`, `docs/guide/getting-started.html` | **Required** |
| Replay snapshots (`src/persist.rs`, `src/app.rs` replay_*) | `docs/guide/replay.html` | **Required** |
| `Cargo.toml` features | `docs/guide/advanced-stats.html`, landing-page copy | **Required** |
| New / renamed source module, changed build command | `docs/guide/development.html` | **Required** |
| Pure internal refactor with no user-visible effects | (nothing) | `refactor:` / `chore:` |

When you add or remove a guide page:

- Update the sidebar in **every** file under `docs/guide/` — the sidebar is
  duplicated by design, not generated.
- Update the `docs/index.html` top nav and any feature copy that references
  the page.
- Fix the `<div class="pager">` links on the previous and next pages.

## PR checklist Copilot should self-apply

Before finalizing a change, verify:

- [ ] Code compiles (`cargo build`).
- [ ] Tests pass on both feature configurations (`cargo test` and
      `cargo test --features advanced-stats`).
- [ ] Format and clippy pass (`cargo fmt --check`,
      `cargo clippy -- -D warnings`, `cargo clippy --features advanced-stats -- -D warnings`).
- [ ] If user-visible: `docs/` is updated and all sidebar/pager links still work.
- [ ] **Commit subject uses a [Conventional Commit](https://www.conventionalcommits.org/)
      prefix** (`feat:` / `fix:` / `feat!:` / `perf:` for user-visible changes;
      `docs:` / `chore:` / `refactor:` / `test:` / `ci:` / `style:` for
      non-user-visible changes). See
      [Commit & changelog rule](#commit--changelog-rule-mandatory).
- [ ] `CHANGELOG.md`, the `version` in `Cargo.toml`, and the changelog
      section of `docs/guide/development.html` are **not** touched — those
      are owned by `release-plz`.
- [ ] `README.md` stays minimal — new material goes in `docs/`, not the README.
- [ ] No new network calls, no new runtime dependencies on external services.
- [ ] No build tooling introduced under `docs/`.

## Commit style

- One topic per commit.
- **[Conventional Commit](https://www.conventionalcommits.org/) subjects are
  required.** `feat(scoring): add balk reason to runner advance`,
  `fix(persist): sanitize colons on Windows`, `docs: document balk reason`.
  See the [Commit & changelog rule](#commit--changelog-rule-mandatory) for
  the full table of prefixes.
- Always include the docs update in the same PR as the code change; a PR
  description should mention both.

## When the agent is unsure

If Copilot is unsure whether a change counts as "user-visible" and warrants a
docs update **and a `feat`/`fix`/`feat!`/`perf` commit prefix**, the answer
is **yes**. A paragraph of HTML and the right commit prefix are cheap;
leaving the site or the release notes out of date is expensive.

A PR that changes behavior but does not use a release-triggering Conventional
Commit prefix, or does not touch `docs/`, should be treated as incomplete and
revised before merge.
