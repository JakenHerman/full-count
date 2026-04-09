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
import textwrap
import urllib.request
from pathlib import Path

MLB_TEAM_ID_ASTROS = 117
MLB_TEAM_NAME_ASTROS = "Houston Astros"

# Event types currently representable in full-count:
# - At-bat outcomes (hit, out, walk/HBP, error, FC, DP, SF)
# - Manual runner advancements (SB/CS/WP/PB/Balk)
SUPPORTED_EVENT_TYPES = {
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


def classify_unsupported_events(all_plays: list[dict]) -> list[dict]:
    unsupported = []
    for play in all_plays:
        result = play.get("result", {})
        event_type = result.get("eventType")
        if not event_type:
            continue
        if event_type in SUPPORTED_EVENT_TYPES:
            continue
        unsupported.append(
            {
                "play_id": play.get("playId"),
                "inning": play.get("about", {}).get("inning"),
                "half": "top" if play.get("about", {}).get("isTopInning") else "bottom",
                "event_type": event_type,
                "event": result.get("event"),
                "description": result.get("description"),
            }
        )
    return unsupported


def issue_body(game: dict, unsupported: dict, fixture_path: Path) -> str:
    return textwrap.dedent(
        f"""\
        ## Summary
        While validating Astros game scoreability for `{game['official_date']}`, full-count found an event that is not currently representable.

        ## Fixture
        - Path: `{fixture_path}`
        - Game PK: `{game['game_pk']}`
        - Opponent: `{game['opponent']}`
        - Status: `{game['status']}`

        ## Unsupported event
        - Inning: `{unsupported['inning']}` (`{unsupported['half']}`)
        - MLB eventType: `{unsupported['event_type']}`
        - MLB event label: `{unsupported.get('event')}`
        - Description: `{unsupported.get('description')}`
        - playId: `{unsupported.get('play_id')}`

        ## Why this blocks full scoring
        This play type does not map to an existing full-count scoring input (`B/S/F`, at-bat outcomes, or manual runner-advance reasons). The game cannot be scored end-to-end without either lossy approximation or manual out-of-band notes.

        ## Proposed fix
        - Add explicit handling for MLB event type `{unsupported['event_type']}`.
        - Define how it updates outs, runners, RBI attribution, AB eligibility, and pitcher/batter stats.
        - Add regression tests using this exact play scenario.
        """
    )


def write_issue_markdown(issue_dir: Path, game: dict, unsupported_events: list[dict], fixture_path: Path) -> list[Path]:
    issue_dir.mkdir(parents=True, exist_ok=True)
    written = []
    for idx, unsupported in enumerate(unsupported_events, start=1):
        issue_path = issue_dir / (
            f"astros-{game['official_date']}-game-{game['game_pk']}-event-{idx}.md"
        )
        issue_path.write_text(issue_body(game, unsupported, fixture_path), encoding="utf-8")
        written.append(issue_path)
    return written


def run(date_str: str, team_id: int, team_name: str, out_dir: Path) -> Path:
    schedule = fetch_json(schedule_url(team_id, date_str))
    games = []
    total_unsupported = 0

    for day in schedule.get("dates", []):
        for game in day.get("games", []):
            game_pk = game["gamePk"]
            feed = fetch_json(game_feed_url(game_pk))
            linescore = feed.get("liveData", {}).get("linescore", {})
            teams_ls = linescore.get("teams", {})

            astros_is_home = (
                game.get("teams", {}).get("home", {}).get("team", {}).get("id") == team_id
            )
            astros_key = "home" if astros_is_home else "away"
            opp_key = "away" if astros_is_home else "home"
            astros_stats = teams_ls.get(astros_key, {})
            opp_stats = teams_ls.get(opp_key, {})

            all_plays = feed.get("liveData", {}).get("plays", {}).get("allPlays", [])
            unsupported_events = classify_unsupported_events(all_plays)
            total_unsupported += len(unsupported_events)

            games.append(
                {
                    "game_pk": game_pk,
                    "official_date": game.get("officialDate"),
                    "status": game.get("status", {}).get("detailedState"),
                    "astros_home": astros_is_home,
                    "opponent": game.get("teams", {}).get(opp_key, {}).get("team", {}).get("name"),
                    "astros_runs": astros_stats.get("runs"),
                    "astros_hits": astros_stats.get("hits"),
                    "astros_errors": astros_stats.get("errors"),
                    "opponent_runs": opp_stats.get("runs"),
                    "opponent_hits": opp_stats.get("hits"),
                    "opponent_errors": opp_stats.get("errors"),
                    "plays_total": len(all_plays),
                    "unsupported_events": unsupported_events,
                }
            )

    out = {
        "pulled_at_utc": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat(),
        "date": date_str,
        "team_id": team_id,
        "team_name": team_name,
        "schedule_url": schedule_url(team_id, date_str),
        "games": games,
        "total_games": len(games),
        "total_unsupported_events": total_unsupported,
    }

    out_dir.mkdir(parents=True, exist_ok=True)
    fixture_path = out_dir / f"astros-scoreability-{date_str}.json"
    fixture_path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")

    issue_dir = out_dir / "issues" / date_str
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
    parser.add_argument("--out-dir", default="test_data/mlb")
    args = parser.parse_args()

    run(
        date_str=args.date,
        team_id=args.team_id,
        team_name=args.team_name,
        out_dir=Path(args.out_dir),
    )


if __name__ == "__main__":
    main()
