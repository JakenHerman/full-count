## Summary
While validating Astros game scoreability for `2026-04-08`, full-count found `1` occurrence(s) of an event type that is not currently representable.

## Fixture
- Path: `/workspace/test_data/mlb/astros-scoreability-2026-04-09-game-824374.json`
- Game PK: `824374`
- Opponent: `Colorado Rockies`
- Status: `Final`

## Unsupported event
- MLB eventType: `sac_bunt`
- Sample event label: `Sac Bunt`
- Sample playId: `None`
- Why unscoreable: `full-count has no sacrifice-bunt at-bat result. Approximating as a groundout would incorrectly count an at-bat and miss SH-specific scoring semantics.`

## Occurrences in this game
- Inning `2` (bottom): `Sac Bunt` — Tyler Freeman out on a sacrifice bunt, catcher Christian Vázquez to first baseman Christian Walker. Kyle Karros to 3rd. Edouard Julien to 2nd.

## Why this blocks full scoring
This play type does not map to an existing full-count scoring input (`B/S/F`, at-bat outcomes, or manual runner-advance reasons). The game cannot be scored end-to-end without either lossy approximation or manual out-of-band notes.

## Proposed fix
- Add explicit handling for MLB event type `sac_bunt`.
- Define how it updates outs, runners, RBI attribution, AB eligibility, and pitcher/batter stats.
- Add regression tests using this exact play scenario.
