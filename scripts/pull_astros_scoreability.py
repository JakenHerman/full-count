#!/usr/bin/env python3
"""Pull Astros game stats from MLB Stats API and generate scoreability fixtures.

This script snapshots Astros games for a given date, then classifies every
`allPlays[].result.eventType` as either currently scoreable in full-count or
unsupported. Unsupported events are exported both in the JSON fixture and as
issue-ready markdown files.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import shutil
import textwrap
import urllib.request
from pathlib import Path

MLB_TEAM_ID_ASTROS = 117
MLB_TEAM_NAME_ASTROS = "Houston Astros"

# Event types currently representable in full-count:
# - At-bat outcomes (hit, out, walk/HBP, error, FC, DP, SF)
# - Manual runner advancements (SB/CS/WP/PB/Balk)
DIRECT_SUPPORTED_EVENT_TYPES = {
    "single",
    "double",
    "triple",
    "home_run",
    "strikeout",
    "walk",
    "intent_walk",
    "hit_by_pitch",
    "groundout",
    "flyout",
    "lineout",
    "pop_out",
    "force_out",
    "field_error",
    "fielder_choice",
    "fielders_choice_out",
    "double_play",
    "grounded_into_double_play",
    "sac_fly",
    "sac_fly_double_play",
    "stolen_base",
    "caught_stealing",
    "wild_pitch",
    "passed_ball",
    "balk",
}

# MLB events that can be represented in full-count by mapping them onto the
# existing scoring vocabulary.
FIELD_OUT_EVENT_LABEL_MAP = {
    "groundout": "groundout",
    "flyout": "flyout",
    "lineout": "flyout",
    "pop_out": "flyout",
}


def fetch_json(url: str) -> dict:
    with urllib.request.urlopen(url, timeout=30) as response:
        return json.loads(response.read().decode("utf-8"))


def schedule_url(team_id: int, date_str: str) -> str:
    return (
        "https://statsapi.mlb.com/api/v1/schedule"
        f"?sportId=1&teamId={team_id}&date={date_str}"
    )


def game_feed_url(game_pk: int) -> str:
    return f"https://statsapi.mlb.com/api/v1.1/game/{game_pk}/feed/live"


def normalize_label(label: str | None) -> str:
    if not label:
        return ""
    return (
        label.strip()
        .lower()
        .replace(" ", "_")
        .replace("-", "_")
        .replace("__", "_")
    )


def scoreability_for_event(event_type: str | None, event_label: str | None) -> tuple[bool, str]:
    if not event_type:
        return True, "missing_event_type"
    if event_type in DIRECT_SUPPORTED_EVENT_TYPES:
        return True, f"direct:{event_type}"
    label_norm = normalize_label(event_label)
    if event_type == "field_out" and label_norm in FIELD_OUT_EVENT_LABEL_MAP:
        mapped = FIELD_OUT_EVENT_LABEL_MAP[label_norm]
        return True, f"mapped:field_out->{mapped}"
    if event_type.startswith("pickoff_caught_stealing_"):
        return True, "mapped:pickoff_caught_stealing->caught_stealing"
    return False, "unmapped"


def unscoreable_reason(event_type: str | None, event_label: str | None) -> str:
    if event_type == "sac_bunt":
        return (
            "full-count has no sacrifice-bunt at-bat result. Approximating as a groundout "
            "would incorrectly count an at-bat and miss SH-specific scoring semantics."
        )
    return (
        "No direct or mapped full-count action exists for this MLB event type."
    )


def classify_unsupported_events(all_plays: list[dict]) -> tuple[list[dict], int]:
    unsupported = []
    supported = 0
    for play in all_plays:
        result = play.get("result", {})
        event_type = result.get("eventType")
        event_label = result.get("event")
        scoreable, mapping = scoreability_for_event(event_type, event_label)
        if scoreable:
            supported += 1
            continue
        unsupported.append(
            {
                "play_id": play.get("playId"),
                "inning": play.get("about", {}).get("inning"),
                "half": "top" if play.get("about", {}).get("isTopInning") else "bottom",
                "event_type": event_type,
                "event": event_label,
                "description": result.get("description"),
                "mapping": mapping,
                "why_unscoreable": unscoreable_reason(event_type, event_label),
            }
        )
    return unsupported, supported


def issue_body(game: dict, event_type: str, occurrences: list[dict], fixture_path: Path) -> str:
    sample = occurrences[0]
    details = "\n".join(
        [
            (
                f"- Inning `{item['inning']}` ({item['half']}): "
                f"`{item.get('event')}` — {item.get('description')}"
            )
            for item in occurrences
        ]
    )
    return textwrap.dedent(
        f"""\
        ## Summary
        While validating Astros game scoreability for `{game['official_date']}`, full-count found `{len(occurrences)}` occurrence(s) of an event type that is not currently representable.

        ## Fixture
        - Path: `{fixture_path}`
        - Game PK: `{game['game_pk']}`
        - Opponent: `{game['opponent']}`
        - Status: `{game['status']}`

        ## Unsupported event
        - MLB eventType: `{event_type}`
        - Sample event label: `{sample.get('event')}`
        - Sample playId: `{sample.get('play_id')}`
        - Why unscoreable: `{sample.get('why_unscoreable')}`

        ## Occurrences in this game
        {details}

        ## Why this blocks full scoring
        This play type does not map to an existing full-count scoring input (`B/S/F`, at-bat outcomes, or manual runner-advance reasons). The game cannot be scored end-to-end without either lossy approximation or manual out-of-band notes.

        ## Proposed fix
        - Add explicit handling for MLB event type `{event_type}`.
        - Define how it updates outs, runners, RBI attribution, AB eligibility, and pitcher/batter stats.
        - Add regression tests using this exact play scenario.
        """
    )


def write_issue_markdown(
    issue_dir: Path, game: dict, unsupported_events: list[dict], fixture_path: Path
) -> list[Path]:
    issue_dir.mkdir(parents=True, exist_ok=True)
    by_type: dict[str, list[dict]] = {}
    for unsupported in unsupported_events:
        key = unsupported.get("event_type") or "unknown_event_type"
        by_type.setdefault(key, []).append(unsupported)
    written = []
    for event_type, occurrences in sorted(by_type.items()):
        slug = normalize_label(event_type) or "unknown_event_type"
        issue_path = issue_dir / (
            f"astros-{game['official_date']}-game-{game['game_pk']}-event-type-{slug}.md"
        )
        issue_path.write_text(
            issue_body(game, event_type, occurrences, fixture_path), encoding="utf-8"
        )
        written.append(issue_path)
    return written


def game_from_feed(feed: dict, team_id: int) -> dict:
    game_data = feed.get("gameData", {})
    live_data = feed.get("liveData", {})
    linescore = live_data.get("linescore", {})
    teams_ls = linescore.get("teams", {})
    home_team = game_data.get("teams", {}).get("home", {})
    away_team = game_data.get("teams", {}).get("away", {})
    astros_is_home = home_team.get("id") == team_id
    astros_key = "home" if astros_is_home else "away"
    opp_key = "away" if astros_is_home else "home"
    astros_stats = teams_ls.get(astros_key, {})
    opp_stats = teams_ls.get(opp_key, {})
    all_plays = live_data.get("plays", {}).get("allPlays", [])
    unsupported_events, supported_events = classify_unsupported_events(all_plays)
    return {
        "game_pk": game_data.get("game", {}).get("pk"),
        "official_date": game_data.get("datetime", {}).get("officialDate"),
        "status": game_data.get("status", {}).get("detailedState"),
        "astros_home": astros_is_home,
        "opponent": (away_team if astros_is_home else home_team).get("name"),
        "astros_runs": astros_stats.get("runs"),
        "astros_hits": astros_stats.get("hits"),
        "astros_errors": astros_stats.get("errors"),
        "opponent_runs": opp_stats.get("runs"),
        "opponent_hits": opp_stats.get("hits"),
        "opponent_errors": opp_stats.get("errors"),
        "plays_total": len(all_plays),
        "supported_events": supported_events,
        "unsupported_events": unsupported_events,
    }


def run(
    query_date: str,
    team_id: int,
    team_name: str,
    out_dir: Path,
    game_pk: int | None = None,
) -> Path:
    schedule = fetch_json(schedule_url(team_id, query_date))
    games = []
    total_unsupported = 0
    total_supported = 0

    if game_pk is not None:
        feed = fetch_json(game_feed_url(game_pk))
        game = game_from_feed(feed, team_id)
        games.append(game)
        total_unsupported += len(game["unsupported_events"])
        total_supported += game["supported_events"]
    else:
        for day in schedule.get("dates", []):
            for game_row in day.get("games", []):
                feed = fetch_json(game_feed_url(game_row["gamePk"]))
                game = game_from_feed(feed, team_id)
                games.append(game)
                total_unsupported += len(game["unsupported_events"])
                total_supported += game["supported_events"]

    out = {
        "pulled_at_utc": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat(),
        "query_date": query_date,
        "team_id": team_id,
        "team_name": team_name,
        "schedule_url": schedule_url(team_id, query_date),
        "game_pk_filter": game_pk,
        "games": games,
        "total_games": len(games),
        "total_supported_events": total_supported,
        "total_unsupported_events": total_unsupported,
    }

    out_dir.mkdir(parents=True, exist_ok=True)
    if game_pk is None:
        fixture_name = f"astros-scoreability-{query_date}.json"
    else:
        fixture_name = f"astros-scoreability-{query_date}-game-{game_pk}.json"
    fixture_path = out_dir / fixture_name
    fixture_path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")

    issue_dir = out_dir / "issues" / query_date
    if issue_dir.exists():
        shutil.rmtree(issue_dir)
    issue_count = 0
    for game in games:
        written = write_issue_markdown(issue_dir, game, game["unsupported_events"], fixture_path)
        issue_count += len(written)

    print(f"wrote fixture: {fixture_path}")
    print(f"games: {len(games)}")
    print(f"unsupported events: {total_unsupported}")
    print(f"issue drafts: {issue_count}")
    return fixture_path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--date", default=dt.date.today().isoformat(), help="YYYY-MM-DD")
    parser.add_argument("--team-id", type=int, default=MLB_TEAM_ID_ASTROS)
    parser.add_argument("--team-name", default=MLB_TEAM_NAME_ASTROS)
    parser.add_argument("--game-pk", type=int, default=None)
    parser.add_argument("--out-dir", default="test_data/mlb")
    args = parser.parse_args()

    run(
        query_date=args.date,
        team_id=args.team_id,
        team_name=args.team_name,
        out_dir=Path(args.out_dir),
        game_pk=args.game_pk,
    )


if __name__ == "__main__":
    main()
