use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, AppScreen, FielderResultType, InputMode, LineupField, SetupSection};
use crate::game::{GameState, Half, TeamColor};

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        AppScreen::Title => draw_title(f, app),
        AppScreen::Setup => draw_setup(f, app),
        AppScreen::Scoring => draw_scoring(f, app),
        AppScreen::Summary => draw_summary(f, app),
        AppScreen::LoadGame => draw_load_screen(f, app),
        AppScreen::Replay => draw_replay(f, app),
    }
}

// ── Title Screen ───────────────────────────────────────────────────────────────

fn draw_title(f: &mut Frame, app: &App) {
    let area = f.area();

    // Clear background
    f.render_widget(Clear, area);

    let logo: Vec<&str> = vec![
        r"  _____ _   _ _     _       ____ ___  _   _ _   _ _____ ",
        r" |  ___| | | | |   | |     / ___/ _ \| | | | \ | |_   _|",
        r" | |_  | | | | |   | |    | |  | | | | | | |  \| | | |  ",
        r" |  _| | |_| | |___| |___ | |__| |_| | |_| | |\  | | |  ",
        r" |_|    \___/|_____|_____| \____\___/ \___/|_| \_| |_|  ",
    ];

    let logo_height = logo.len() as u16;
    let menu_items = ["  New Game  ", "  Load Game ", "  Quit      "];
    let menu_height = menu_items.len() as u16;
    // logo + spacer + baseball + spacer + menu + spacer + footer
    let total_height = logo_height + 1 + 1 + 1 + menu_height + 1 + 1;
    let top_pad = area.height.saturating_sub(total_height) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),     // top padding
            Constraint::Length(logo_height), // ASCII logo
            Constraint::Length(1),           // spacer
            Constraint::Length(1),           // baseball decoration
            Constraint::Length(1),           // spacer
            Constraint::Length(menu_height), // menu
            Constraint::Length(1),           // spacer
            Constraint::Length(1),           // footer
            Constraint::Min(0),              // bottom fill
        ])
        .split(area);

    // ── Logo ────────────────────────────────────────────────────────────────────
    let logo_lines: Vec<Line> = logo
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                *l,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    f.render_widget(
        Paragraph::new(logo_lines).alignment(ratatui::layout::Alignment::Center),
        chunks[1],
    );

    // ── Baseball decoration ─────────────────────────────────────────────────────
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "\u{26be} Every pitch. Every play. Every out. \u{26be}",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(ratatui::layout::Alignment::Center),
        chunks[3],
    );

    // ── Menu items ─────────────────────────────────────────────────────────────
    let menu_lines: Vec<Line> = menu_items
        .iter()
        .enumerate()
        .map(|(i, label)| {
            if i == app.title_cursor as usize {
                Line::from(vec![
                    Span::styled("  \u{276f} ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        *label,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(*label, Style::default().fg(Color::White)),
                ])
            }
        })
        .collect();
    f.render_widget(
        Paragraph::new(menu_lines).alignment(ratatui::layout::Alignment::Center),
        chunks[5],
    );

    // ── Footer ──────────────────────────────────────────────────────────────────
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "[\u{2191}\u{2193}/J/K] select   [Enter] confirm   [Ctrl+C] quit",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(ratatui::layout::Alignment::Center),
        chunks[7],
    );
}

// ── Setup Screen ───────────────────────────────────────────────────────────

fn draw_setup(f: &mut Frame, app: &App) {
    let area = f.area();

    let outer = Block::default()
        .title(" Full Count \u{2014} Game Setup ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(outer, area);

    let inner = inner_area(area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // team name fields
            Constraint::Length(3), // team color pickers
            Constraint::Length(1), // spacer
            Constraint::Length(1), // column headers
            Constraint::Length(9), // 9 lineup rows
            Constraint::Length(1), // spacer
            Constraint::Length(3), // starter fields
            Constraint::Min(1),    // footer / status
        ])
        .split(inner);

    // ── Team name fields ───────────────────────────────────────────────────
    let name_cols = two_col_split(rows[0]);
    render_input_field(
        f,
        " Away Team ",
        &app.setup.away_name,
        app.setup_cursor == SetupSection::AwayTeamName,
        name_cols[0],
    );
    render_input_field(
        f,
        " Home Team ",
        &app.setup.home_name,
        app.setup_cursor == SetupSection::HomeTeamName,
        name_cols[1],
    );

    // ── Team color pickers ─────────────────────────────────────────────────
    let color_cols = two_col_split(rows[1]);
    render_color_picker(
        f,
        " Away Color ",
        app.setup.away_color,
        app.setup_cursor == SetupSection::AwayColor,
        color_cols[0],
    );
    render_color_picker(
        f,
        " Home Color ",
        app.setup.home_color,
        app.setup_cursor == SetupSection::HomeColor,
        color_cols[1],
    );

    // ── Column headers ─────────────────────────────────────────────────────
    let hdr_cols = two_col_split(rows[3]);
    let hdr_style = Style::default().fg(Color::DarkGray);
    let hdr_line = if cfg!(feature = "advanced-stats") {
        Line::from(vec![
            Span::styled("  #  ", hdr_style),
            Span::styled(format!("{:<18}", "Name"), hdr_style),
            Span::styled("  Avg", hdr_style),
        ])
    } else {
        Line::from(vec![
            Span::styled("  #  ", hdr_style),
            Span::styled(format!("{:<18}", "Name"), hdr_style),
        ])
    };
    f.render_widget(Paragraph::new(hdr_line.clone()), hdr_cols[0]);
    f.render_widget(Paragraph::new(hdr_line), hdr_cols[1]);

    // ── Lineup rows ────────────────────────────────────────────────────────
    let lineup_area = rows[4];
    for i in 0..9 {
        let row_rect = Rect {
            x: lineup_area.x,
            y: lineup_area.y + i as u16,
            width: lineup_area.width,
            height: 1,
        };
        let halves = two_col_split(row_rect);

        render_lineup_row(
            f,
            i + 1,
            &app.setup.away_lineup[i].name,
            &app.setup.away_lineup[i].avg,
            app.setup_cursor == SetupSection::AwayLineup(i, LineupField::Name),
            app.setup_cursor == SetupSection::AwayLineup(i, LineupField::Avg),
            halves[0],
        );
        render_lineup_row(
            f,
            i + 1,
            &app.setup.home_lineup[i].name,
            &app.setup.home_lineup[i].avg,
            app.setup_cursor == SetupSection::HomeLineup(i, LineupField::Name),
            app.setup_cursor == SetupSection::HomeLineup(i, LineupField::Avg),
            halves[1],
        );
    }

    // ── Starter fields ─────────────────────────────────────────────────────
    let starter_cols = two_col_split(rows[6]);
    render_input_field(
        f,
        " Away Starter (P) ",
        &app.setup.away_starter,
        app.setup_cursor == SetupSection::AwayStarter,
        starter_cols[0],
    );
    render_input_field(
        f,
        " Home Starter (P) ",
        &app.setup.home_starter,
        app.setup_cursor == SetupSection::HomeStarter,
        starter_cols[1],
    );

    // ── Footer ─────────────────────────────────────────────────────────────
    let footer = if let Some(ref msg) = app.status_message {
        Line::from(Span::styled(msg.as_str(), Style::default().fg(Color::Red)))
    } else {
        Line::from(Span::styled(
            " [Tab] next field   [Shift+Tab] prev   [Enter] start game   [F3] load save   [Ctrl+C] quit",
            Style::default().fg(Color::DarkGray),
        ))
    };
    f.render_widget(Paragraph::new(footer), rows[7]);

    // ── Cursor positioning ─────────────────────────────────────────────────
    let (cx, cy) = setup_cursor_pos(app, &name_cols, &color_cols, lineup_area, &starter_cols);
    if cx < area.x + area.width && cy < area.y + area.height {
        f.set_cursor_position((cx, cy));
    }
}

fn setup_cursor_pos(
    app: &App,
    name_cols: &[Rect],
    color_cols: &[Rect],
    lineup_area: Rect,
    starter_cols: &[Rect],
) -> (u16, u16) {
    let val = app.current_setup_field_value();
    let vlen = val.len() as u16;

    match &app.setup_cursor {
        SetupSection::AwayTeamName => (name_cols[0].x + 1 + vlen, name_cols[0].y + 1),
        SetupSection::HomeTeamName => (name_cols[1].x + 1 + vlen, name_cols[1].y + 1),
        // Color fields: hide the text cursor by placing it inside the block
        SetupSection::AwayColor => (color_cols[0].x + 1, color_cols[0].y + 1),
        SetupSection::HomeColor => (color_cols[1].x + 1, color_cols[1].y + 1),
        SetupSection::AwayStarter => (starter_cols[0].x + 1 + vlen, starter_cols[0].y + 1),
        SetupSection::HomeStarter => (starter_cols[1].x + 1 + vlen, starter_cols[1].y + 1),
        // Lineup: row has " N  {name:<18}  {avg}" — name starts at col 5, avg at col 25
        SetupSection::AwayLineup(i, LineupField::Name) => {
            (lineup_area.x + 5 + vlen, lineup_area.y + *i as u16)
        }
        SetupSection::AwayLineup(i, LineupField::Avg) => {
            (lineup_area.x + 25 + vlen, lineup_area.y + *i as u16)
        }
        SetupSection::HomeLineup(i, LineupField::Name) => {
            let half_x = lineup_area.x + lineup_area.width / 2;
            (half_x + 5 + vlen, lineup_area.y + *i as u16)
        }
        SetupSection::HomeLineup(i, LineupField::Avg) => {
            let half_x = lineup_area.x + lineup_area.width / 2;
            (half_x + 25 + vlen, lineup_area.y + *i as u16)
        }
    }
}

fn render_color_picker(f: &mut Frame, title: &str, color: TeamColor, focused: bool, area: Rect) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let rcolor = color.to_color();
    let swatch = "\u{2588}\u{2588}\u{2588}";
    let content = if focused {
        Line::from(vec![
            Span::styled(" \u{25c0} ", Style::default().fg(Color::Yellow)),
            Span::styled(swatch, Style::default().fg(rcolor)),
            Span::styled(
                format!(" {} ", color.name()),
                Style::default().fg(rcolor).add_modifier(Modifier::BOLD),
            ),
            Span::styled(swatch, Style::default().fg(rcolor)),
            Span::styled(" \u{25b6}", Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(vec![Span::styled(
            format!("   {} {} {}", swatch, color.name(), swatch),
            Style::default().fg(rcolor),
        )])
    };

    f.render_widget(Paragraph::new(content).block(block), area);
}

fn render_input_field(f: &mut Frame, title: &str, value: &str, focused: bool, area: Rect) {
    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style);
    f.render_widget(Paragraph::new(value).block(block), area);
}

fn render_lineup_row(
    f: &mut Frame,
    num: usize,
    name: &str,
    avg: &str,
    name_focused: bool,
    avg_focused: bool,
    area: Rect,
) {
    let name_style = if name_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let line = if cfg!(feature = "advanced-stats") {
        let avg_style = if avg_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };
        Line::from(vec![
            Span::styled(
                format!(" {:>1}  ", num),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(format!("{:<18}", name), name_style),
            Span::styled("  ", Style::default()),
            Span::styled(avg.to_string(), avg_style),
        ])
    } else {
        let _ = (avg, avg_focused); // suppress unused warnings when feature is off
        Line::from(vec![
            Span::styled(
                format!(" {:>1}  ", num),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(format!("{:<18}", name), name_style),
        ])
    };
    f.render_widget(Paragraph::new(line), area);
}

// ── Scoring Screen ─────────────────────────────────────────────────────────

fn draw_scoring(f: &mut Frame, app: &App) {
    let Ok(game) = app.game() else { return };
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // line score
            Constraint::Length(10), // diamond + batter/pitcher info
            Constraint::Min(5),     // play log
            Constraint::Length(3),  // footer / keybindings
        ])
        .split(area);

    draw_line_score(f, game, chunks[0]);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    draw_diamond(f, game, mid[0]);
    draw_at_bat_info(f, game, mid[1]);
    draw_play_log(f, game, app.play_log_scroll, chunks[2]);
    draw_scoring_footer(f, app, chunks[3]);

    // Popups render on top of everything else
    draw_popups(f, app, area);
}

fn draw_line_score(f: &mut Frame, game: &GameState, area: Rect) {
    let innings = game.inning_scores.len().max(9);
    let half_arrow = if game.half == Half::Top {
        "\u{25b2}"
    } else {
        "\u{25bc}"
    };

    let header_cells: Vec<Cell> = std::iter::once(Cell::from(""))
        .chain(
            (1..=innings)
                .map(|i| Cell::from(i.to_string()).style(Style::default().fg(Color::DarkGray))),
        )
        .chain([
            Cell::from("R").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("H").style(Style::default().fg(Color::DarkGray)),
            Cell::from("E").style(Style::default().fg(Color::DarkGray)),
        ])
        .collect();

    let header = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD));

    let away_style = if game.half == Half::Top {
        Style::default()
            .fg(game.away.color.to_color())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let home_style = if game.half == Half::Bottom {
        Style::default()
            .fg(game.home.color.to_color())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let away_row = Row::new(score_row_cells(game, false, innings)).style(away_style);
    let home_row = Row::new(score_row_cells(game, true, innings)).style(home_style);

    let mut widths: Vec<Constraint> = vec![Constraint::Length(14)]; // team name
    for _ in 0..innings {
        widths.push(Constraint::Length(3));
    }
    widths.extend([
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(4),
    ]);

    let table = Table::new([away_row, home_row], widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" Inning {} {} ", game.inning, half_arrow))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    f.render_widget(table, area);
}

fn score_row_cells(game: &GameState, is_home: bool, innings: usize) -> Vec<Cell<'static>> {
    let name = if is_home {
        game.home.name.clone()
    } else {
        game.away.name.clone()
    };
    let mut cells: Vec<Cell<'static>> = vec![Cell::from(name)];

    for i in 0..innings {
        let s = if i < game.inning_scores.len() {
            let score = &game.inning_scores[i];
            let r = if is_home {
                score.home_runs
            } else {
                score.away_runs
            };
            r.to_string()
        } else {
            "-".to_string()
        };
        cells.push(Cell::from(s));
    }

    let r = if is_home {
        game.home_total_runs()
    } else {
        game.away_total_runs()
    };
    let h = if is_home {
        game.home_total_hits()
    } else {
        game.away_total_hits()
    };
    let e = if is_home {
        game.errors.home
    } else {
        game.errors.away
    };

    cells.push(
        Cell::from(r.to_string()).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    );
    cells.push(Cell::from(h.to_string()));
    cells.push(Cell::from(e.to_string()));
    cells
}

fn draw_diamond(f: &mut Frame, game: &GameState, area: Rect) {
    let occ = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let emp = Style::default().fg(Color::DarkGray);
    let line_col = Style::default().fg(Color::DarkGray);

    let s2 = if game.bases.second.is_some() {
        occ
    } else {
        emp
    };
    let s3 = if game.bases.third.is_some() { occ } else { emp };
    let s1 = if game.bases.first.is_some() { occ } else { emp };

    let outs_str: String = (0..3)
        .map(|i| {
            if i < game.outs {
                "\u{25cf}"
            } else {
                "\u{25cb}"
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let diamond = vec![
        Line::from(""),
        Line::from(vec![Span::raw("        "), Span::styled("2B", s2)]),
        Line::from(vec![Span::styled("       /  \\", line_col)]),
        Line::from(vec![
            Span::styled("3B", s3),
            Span::raw("          "),
            Span::styled("1B", s1),
        ]),
        Line::from(vec![Span::styled("       \\  /", line_col)]),
        Line::from(vec![
            Span::raw("        "),
            Span::styled("HP", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
    ];

    f.render_widget(
        Paragraph::new(diamond).block(
            Block::default()
                .title(format!(" Outs: {} ", outs_str))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        ),
        area,
    );
}

fn draw_at_bat_info(f: &mut Frame, game: &GameState, area: Rect) {
    let panel_height = if cfg!(feature = "advanced-stats") {
        6
    } else {
        5
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(panel_height),
            Constraint::Length(panel_height),
        ])
        .split(area);

    // ── Batter panel ───────────────────────────────────────────────────────
    let batter = game.batting_team().current_batter();
    let balls_str: String = (0..4)
        .map(|i| {
            if i < game.count.balls {
                "\u{25cf}"
            } else {
                "\u{25cb}"
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let strikes_str: String = (0..3)
        .map(|i| {
            if i < game.count.strikes {
                "\u{25cf}"
            } else {
                "\u{25cb}"
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let batter_name_line = if cfg!(feature = "advanced-stats") {
        Line::from(vec![
            Span::styled("Batter:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                batter.info.name.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", batter.info.avg_display()),
                Style::default().fg(Color::Cyan),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("Batter:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                batter.info.name.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    };

    let mut batter_lines = vec![
        batter_name_line,
        Line::from(vec![
            Span::styled("Balls:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(&balls_str, Style::default().fg(Color::Green)),
            Span::styled("   Strikes: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&strikes_str, Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::raw("         "),
            Span::styled(
                format!(
                    "AB:{} H:{} BB:{} K:{} RBI:{}",
                    batter.stats.at_bats,
                    batter.stats.hits,
                    batter.stats.walks,
                    batter.stats.strikeouts,
                    batter.stats.rbi
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    if cfg!(feature = "advanced-stats") {
        batter_lines.push(Line::from(vec![
            Span::raw("         "),
            Span::styled(
                format!(
                    "2B:{} 3B:{} HR:{} SB:{}",
                    batter.stats.doubles,
                    batter.stats.triples,
                    batter.stats.home_runs,
                    batter.stats.stolen_bases
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    let batter_text = Text::from(batter_lines);

    f.render_widget(
        Paragraph::new(batter_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        ),
        chunks[0],
    );

    // ── Pitcher panel ──────────────────────────────────────────────────────
    let pitcher = game.fielding_team().current_pitcher();
    let mut pitcher_lines = vec![
        Line::from(vec![
            Span::styled("Pitcher: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                pitcher.info.name.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("   PC: {}", pitcher.stats.pitch_count),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("         "),
            Span::styled(
                format!(
                    "IP:{}  H:{}  R:{}  ER:{}",
                    pitcher.stats.ip_display(),
                    pitcher.stats.hits_allowed,
                    pitcher.stats.runs_allowed,
                    pitcher.stats.earned_runs
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    if cfg!(feature = "advanced-stats") {
        pitcher_lines.push(Line::from(vec![
            Span::raw("         "),
            Span::styled(
                format!(
                    "BB:{}  K:{}  WP:{}  BF:{}",
                    pitcher.stats.walks,
                    pitcher.stats.strikeouts,
                    pitcher.stats.wild_pitches,
                    pitcher.stats.batters_faced
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        pitcher_lines.push(Line::from(vec![
            Span::raw("         "),
            Span::styled(
                format!("BB:{}  K:{}", pitcher.stats.walks, pitcher.stats.strikeouts),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    let pitcher_text = Text::from(pitcher_lines);

    f.render_widget(
        Paragraph::new(pitcher_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        ),
        chunks[1],
    );
}

fn draw_play_log(f: &mut Frame, game: &GameState, scroll: usize, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = game.play_log.len();

    let start = if total > visible_height {
        scroll.min(total - visible_height)
    } else {
        0
    };

    let items: Vec<ListItem> = game
        .play_log
        .iter()
        .skip(start)
        .take(visible_height)
        .map(|e| ListItem::new(e.display()))
        .collect();

    let title = if total == 0 {
        " Play Log ".to_string()
    } else {
        format!(
            " Play Log ({}\u{2013}{} of {}) ",
            start + 1,
            (start + visible_height).min(total),
            total
        )
    };

    f.render_widget(
        List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::White)),
        area,
    );
}

fn draw_scoring_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(ref msg) = app.status_message {
        Text::from(Line::from(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(Color::Yellow),
        )))
    } else {
        Text::from(vec![
            Line::from(Span::styled(
                " [B]all [S]trike [F]oul  \u{2502}  [1][2][3][H]it [K]K-swing [L]K-look [W]alk [P]HBP",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                " [G]rdout [D]P [O]flyout [E]rror [C]FC [V]SacFly  \u{2502}  [A]dvance [Tab]Pitcher  [U]ndo  [F2]Save  [X]End [Q]Quit",
                Style::default().fg(Color::DarkGray),
            )),
        ])
    };

    f.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        area,
    );
}

fn draw_popups(f: &mut Frame, app: &App, area: Rect) {
    match &app.input_mode {
        InputMode::FielderInput {
            result_type,
            buffer,
        } => {
            let (title, hint) = match result_type {
                FielderResultType::Groundout => (" Groundout ", "Fielder sequence  e.g. 6-3"),
                FielderResultType::DoublePlay => (" Double Play ", "Sequence  e.g. 6-4-3"),
                FielderResultType::Flyout => (" Flyout ", "Fielder position  e.g. 8"),
                FielderResultType::Error => (" Error ", "Fielder position  e.g. 6"),
                FielderResultType::SacrificeFly => (" Sac Fly ", "Fielder position  e.g. 9"),
            };
            draw_input_popup(
                f,
                area,
                title,
                hint,
                buffer,
                "[Enter] confirm  [Esc] cancel",
            );
        }
        InputMode::RbiInput {
            pending_result,
            buffer,
            ..
        } => {
            let title = format!(" RBIs \u{2014} {} ", pending_result.display());
            draw_input_popup(
                f,
                area,
                &title,
                "Enter 0\u{2013}4",
                buffer,
                "[Enter] confirm  [Esc] cancel",
            );
        }
        InputMode::PitcherChange { name_buffer } => {
            draw_input_popup(
                f,
                area,
                " Pitcher Change ",
                "New pitcher name",
                name_buffer,
                "[Enter] confirm  [Esc] cancel",
            );
        }
        InputMode::SavePrompt { buffer } => {
            draw_input_popup(
                f,
                area,
                " Save Game ",
                "Enter a name for this save",
                buffer,
                "[Enter] save  [Esc] cancel",
            );
        }
        _ => {}
    }
}

fn draw_input_popup(
    f: &mut Frame,
    area: Rect,
    title: &str,
    hint: &str,
    buffer: &str,
    footer: &str,
) {
    let popup = centered_popup(area, 46, 7);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let lines = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(vec![
            Span::styled("  \u{276f} ", Style::default().fg(Color::Yellow)),
            Span::styled(
                buffer,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", footer),
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    f.render_widget(Paragraph::new(lines), inner);

    // Position cursor at end of buffer text
    let cx = inner.x + 4 + buffer.len() as u16;
    let cy = inner.y + 2;
    if cx < area.x + area.width && cy < area.y + area.height {
        f.set_cursor_position((cx, cy));
    }
}

// ── Summary Screen ─────────────────────────────────────────────────────────

fn draw_summary(f: &mut Frame, app: &App) {
    let Ok(game) = app.game() else { return };
    let area = f.area();

    let outer = Block::default()
        .title(" \u{26be} Full Count \u{2014} Final Score ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(outer, area);

    let inner = inner_area(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // line score
            Constraint::Min(11),   // away batting
            Constraint::Min(11),   // home batting
            Constraint::Min(6),    // pitching
            Constraint::Length(1), // footer
        ])
        .split(inner);

    draw_summary_line_score(f, game, chunks[0]);
    draw_summary_batting(f, game, false, chunks[1]);
    draw_summary_batting(f, game, true, chunks[2]);
    draw_summary_pitching(f, game, chunks[3]);

    let footer_text = if let Some(msg) = &app.status_message {
        Line::from(vec![Span::styled(
            msg.clone(),
            Style::default().fg(Color::Green),
        )])
    } else {
        Line::from(vec![
            Span::styled("[E]", Style::default().fg(Color::Yellow)),
            Span::raw(" Export HTML  "),
            Span::styled("[Q]", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit"),
        ])
    };
    f.render_widget(Paragraph::new(footer_text), chunks[4]);
}

fn draw_summary_line_score(f: &mut Frame, game: &GameState, area: Rect) {
    let innings = game.inning_scores.len().max(9);

    let header_cells: Vec<Cell> = std::iter::once(Cell::from("").style(Style::default()))
        .chain(
            (1..=innings)
                .map(|i| Cell::from(i.to_string()).style(Style::default().fg(Color::DarkGray))),
        )
        .chain([
            Cell::from("R").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("H"),
            Cell::from("E"),
        ])
        .collect();

    let header = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD));
    let away_row = Row::new(score_row_cells(game, false, innings));
    let home_row = Row::new(score_row_cells(game, true, innings));

    let mut widths: Vec<Constraint> = vec![Constraint::Length(14)];
    for _ in 0..innings {
        widths.push(Constraint::Length(3));
    }
    widths.extend([
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(4),
    ]);

    f.render_widget(
        Table::new([away_row, home_row], widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            ),
        area,
    );
}

fn draw_summary_batting(f: &mut Frame, game: &GameState, is_home: bool, area: Rect) {
    let team = if is_home { &game.home } else { &game.away };

    if cfg!(feature = "advanced-stats") {
        let header = Row::new([
            "Batter", "AB", "R", "H", "2B", "3B", "HR", "RBI", "BB", "K", "SB", "CS",
        ])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let mut rows: Vec<Row> = team
            .lineup
            .iter()
            .map(|slot| {
                Row::new([
                    slot.info.name.clone(),
                    slot.stats.at_bats.to_string(),
                    slot.stats.runs.to_string(),
                    slot.stats.hits.to_string(),
                    slot.stats.doubles.to_string(),
                    slot.stats.triples.to_string(),
                    slot.stats.home_runs.to_string(),
                    slot.stats.rbi.to_string(),
                    slot.stats.walks.to_string(),
                    slot.stats.strikeouts.to_string(),
                    slot.stats.stolen_bases.to_string(),
                    slot.stats.caught_stealing.to_string(),
                ])
            })
            .collect();

        let totals = Row::new([
            "TOTALS".to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.at_bats as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.runs as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.hits as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.doubles as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.triples as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.home_runs as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.rbi as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.walks as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.strikeouts as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.stolen_bases as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.caught_stealing as u32)
                .sum::<u32>()
                .to_string(),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));
        rows.push(totals);

        // LOB row (team-level)
        let mut lob_cells: Vec<String> = vec![format!("LOB: {}", team.left_on_base)];
        lob_cells.extend(std::iter::repeat_n(String::new(), 11));
        rows.push(Row::new(lob_cells).style(Style::default().fg(Color::DarkGray)));

        let widths = [
            Constraint::Min(18),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
        ];

        f.render_widget(
            Table::new(rows, widths).header(header).block(
                Block::default()
                    .title(format!(" {} Batting ", team.name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(team.color.to_color())),
            ),
            area,
        );
    } else {
        let header = Row::new(["Batter", "AB", "R", "H", "RBI", "BB", "K"]).style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let mut rows: Vec<Row> = team
            .lineup
            .iter()
            .map(|slot| {
                Row::new([
                    slot.info.name.clone(),
                    slot.stats.at_bats.to_string(),
                    slot.stats.runs.to_string(),
                    slot.stats.hits.to_string(),
                    slot.stats.rbi.to_string(),
                    slot.stats.walks.to_string(),
                    slot.stats.strikeouts.to_string(),
                ])
            })
            .collect();

        let totals = Row::new([
            "TOTALS".to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.at_bats as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.runs as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.hits as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.rbi as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.walks as u32)
                .sum::<u32>()
                .to_string(),
            team.lineup
                .iter()
                .map(|s| s.stats.strikeouts as u32)
                .sum::<u32>()
                .to_string(),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));
        rows.push(totals);

        let widths = [
            Constraint::Min(20),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ];

        f.render_widget(
            Table::new(rows, widths).header(header).block(
                Block::default()
                    .title(format!(" {} Batting ", team.name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(team.color.to_color())),
            ),
            area,
        );
    }
}

fn draw_summary_pitching(f: &mut Frame, game: &GameState, area: Rect) {
    let col_count = if cfg!(feature = "advanced-stats") {
        10
    } else {
        8
    };

    let header = if cfg!(feature = "advanced-stats") {
        Row::new(["Pitcher", "IP", "H", "R", "ER", "BB", "K", "WP", "BF", "PC"]).style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Row::new(["Pitcher", "IP", "H", "R", "ER", "BB", "K", "PC"]).style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
    };

    let mut rows: Vec<Row> = Vec::new();

    for (team, team_game) in [(&game.away, false), (&game.home, true)] {
        rows.push(
            Row::new(
                std::iter::once(team.name.clone())
                    .chain(std::iter::repeat_n(String::new(), col_count - 1))
                    .collect::<Vec<_>>(),
            )
            .style(
                Style::default()
                    .fg(team.color.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
        );
        for p in &team.pitchers {
            let name = match p.decision {
                Some(d) => format!("  {} ({})", p.info.name, d.label()),
                None => format!("  {}", p.info.name),
            };
            if cfg!(feature = "advanced-stats") {
                rows.push(Row::new([
                    name,
                    p.stats.ip_display(),
                    p.stats.hits_allowed.to_string(),
                    p.stats.runs_allowed.to_string(),
                    p.stats.earned_runs.to_string(),
                    p.stats.walks.to_string(),
                    p.stats.strikeouts.to_string(),
                    p.stats.wild_pitches.to_string(),
                    p.stats.batters_faced.to_string(),
                    p.stats.pitch_count.to_string(),
                ]));
            } else {
                rows.push(Row::new([
                    name,
                    p.stats.ip_display(),
                    p.stats.hits_allowed.to_string(),
                    p.stats.runs_allowed.to_string(),
                    p.stats.earned_runs.to_string(),
                    p.stats.walks.to_string(),
                    p.stats.strikeouts.to_string(),
                    p.stats.pitch_count.to_string(),
                ]));
            }
        }
        if !team_game {
            rows.push(Row::new(
                vec![""; col_count]
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<_>>(),
            ));
        }
    }

    let widths: Vec<Constraint> = if cfg!(feature = "advanced-stats") {
        vec![
            Constraint::Min(22),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(5),
        ]
    } else {
        vec![
            Constraint::Min(22),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
        ]
    };

    f.render_widget(
        Table::new(rows, widths).header(header).block(
            Block::default()
                .title(" Pitching ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        area,
    );
}

// ── Load Game Screen ───────────────────────────────────────────────────────

fn draw_load_screen(f: &mut Frame, app: &App) {
    let area = f.area();

    let outer = Block::default()
        .title(" Full Count \u{2014} Load Saved Game ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(outer, area);

    let inner = inner_area(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    if app.load_slots.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  No saved games found. Start a new game and press F2 to save.",
                Style::default().fg(Color::DarkGray),
            ))),
            chunks[0],
        );
    } else {
        let items: Vec<ListItem> = app
            .load_slots
            .iter()
            .enumerate()
            .map(|(i, slot)| {
                if i == app.load_cursor {
                    ListItem::new(format!("  \u{276f} {}", slot.display)).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(format!("    {}", slot.display))
                        .style(Style::default().fg(Color::White))
                }
            })
            .collect();

        f.render_widget(
            List::new(items).block(
                Block::default()
                    .title(" Saved Games ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray)),
            ),
            chunks[0],
        );
    }

    let footer_text = if app.load_slots.is_empty() {
        " [Esc] back"
    } else {
        " [\u{2191}\u{2193}/J/K] select   [Enter] load   [R] replay   [Esc] back"
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[1],
    );
}

// ── Replay Screen ─────────────────────────────────────────────────────────

fn draw_replay(f: &mut Frame, app: &App) {
    let Some(game) = app.replay_game() else {
        return;
    };
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // line score
            Constraint::Length(10), // diamond + batter/pitcher info
            Constraint::Min(5),     // play log
            Constraint::Length(3),  // replay footer
        ])
        .split(area);

    draw_line_score(f, game, chunks[0]);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    draw_diamond(f, game, mid[0]);
    draw_at_bat_info(f, game, mid[1]);
    draw_play_log(f, game, 0, chunks[2]);
    draw_replay_footer(f, app, chunks[3]);
}

fn draw_replay_footer(f: &mut Frame, app: &App, area: Rect) {
    let total = app.replay_snapshots.len();
    let pos = app.replay_cursor + 1;

    let position_text = format!(" Step {} of {} ", pos, total);

    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                " \u{25c0} [\u{2190}/H] prev   ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                &position_text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "   [\u{2192}/L] next \u{25b6}",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(Span::styled(
            " [G] first   [Shift+G] last   [Esc/Q] exit replay",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    f.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .title(" \u{25b6} Replay ")
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        area,
    );
}

// ── Layout helpers ─────────────────────────────────────────────────────────

fn inner_area(area: Rect) -> Rect {
    Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

fn two_col_split(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area)
        .to_vec()
}

fn centered_popup(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x: x.min(area.x + area.width.saturating_sub(width)),
        y: y.min(area.y + area.height.saturating_sub(height)),
        width: width.min(area.width),
        height: height.min(area.height),
    }
}
