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

2. **Always add a changelog bullet.** See the dedicated
   [Changelog rule (mandatory)](#changelog-rule-mandatory) section below.
   This is not optional.

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

## Changelog rule (mandatory)

**Every PR that changes user-visible behavior must add at least one bullet to
the changelog, in the same PR.** This includes — but is not limited to —
new or changed keybindings, new CLI flags, new prompts, new save-file fields,
renamed commands, new `advanced-stats` metrics, changed defaults, altered error
messages, and any modification to the HTML scorecard output.

**Location.** The changelog lives under the **Unreleased** heading in
[`docs/guide/development.html#changelog`](../docs/guide/development.html#changelog).
There is no `CHANGELOG.md` — that file does not exist and must not be created;
the site is the source of truth.

**Format.** Add a new `<li>` inside the `<ul>` that immediately follows the
`<h3>Unreleased</h3>` heading. One sentence. Past tense. Plain prose, no emoji,
no ticket numbers unless they add real context.

**Example.** Suppose you add a balk reason to the manual runner-advance flow.
The diff to `docs/guide/development.html` must include something like:

```html
<h3>Unreleased</h3>
<ul>
  <li>Added <code>B</code> as a balk reason in the manual runner-advance flow.</li>
  <!-- …existing bullets… -->
</ul>
```

**If the Unreleased section does not exist,** create it above the most recent
version heading:

```html
<h3>Unreleased</h3>
<ul>
  <li>Your bullet here.</li>
</ul>

<h3>0.1.0</h3>
<!-- …existing release notes… -->
```

**When a release ships,** rename `Unreleased` to the new version number
(e.g. `<h3>0.2.0</h3>`) and bump `version` in `Cargo.toml` accordingly. After
the bump, a fresh empty `Unreleased` section stays at the top so the next
change has somewhere to land.

**No escape hatches.** "Internal refactor only" is the one legitimate reason to
skip the changelog — and only if the change produces zero user-visible
differences (same keystrokes, same output, same saved JSON, same CLI help
text). When in doubt, add the bullet.

## Docs-with-code mapping

Every row below also requires a changelog bullet under **Unreleased** in
`docs/guide/development.html#changelog` — that column is implicit and
non-negotiable.

| If you touch… | Then also update at minimum… | Changelog bullet? |
|---------------|------------------------------|-------------------|
| `src/input.rs` (keybindings) | `docs/guide/scoring.html`, landing-page keygrid in `docs/index.html` | **Required** |
| `src/main.rs` (CLI flags) | `docs/guide/getting-started.html`, `docs/guide/saves.html` | **Required** |
| `src/persist.rs` (paths, filenames, sanitization) | `docs/guide/saves.html` | **Required** |
| `src/export.rs` or `templates/scorecard.html` | `docs/guide/saves.html` (HTML scorecard export) | **Required** |
| `src/game.rs` rules (outs, RBI, advancement, substitutions) | `docs/guide/scoring.html`, `docs/guide/getting-started.html` | **Required** |
| Replay snapshots (`src/persist.rs`, `src/app.rs` replay_*) | `docs/guide/replay.html` | **Required** |
| `Cargo.toml` features | `docs/guide/advanced-stats.html`, landing-page copy | **Required** |
| New / renamed source module, changed build command | `docs/guide/development.html` | **Required** |
| Pure internal refactor with no user-visible effects | (nothing) | Not required |

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
- [ ] **If user-visible: a new `<li>` was added under the `<h3>Unreleased</h3>`
      block in `docs/guide/development.html`.** If there is no Unreleased
      block, create one. Do NOT rely on a separate `CHANGELOG.md` — none
      exists. See the [Changelog rule (mandatory)](#changelog-rule-mandatory).
- [ ] The changelog bullet is in the same commit / PR as the code change, not
      deferred to a follow-up.
- [ ] `README.md` stays minimal — new material goes in `docs/`, not the README.
- [ ] No new network calls, no new runtime dependencies on external services.
- [ ] No build tooling introduced under `docs/`.

## Commit style

- One topic per commit.
- Subject lines like `scoring: add balk reason to runner advance` or
  `docs: document balk reason in scoring reference`.
- Always include the docs update in the same PR as the code change; a PR
  description should mention both.

## When the agent is unsure

If Copilot is unsure whether a change counts as "user-visible" and warrants a
docs update **and a changelog bullet**, the answer is **yes**. Writing a
paragraph of HTML and a one-line bullet is cheap; leaving the site or the
changelog out of date is expensive.

A PR that changes behavior but does not touch `docs/guide/development.html`'s
Unreleased list should be treated as incomplete and revised before merge.
