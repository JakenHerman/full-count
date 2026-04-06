mod app;
mod game;
mod input;
mod persist;
mod ui;

use std::io::{self, stdout};
use std::panic;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppScreen};

// ── CLI ────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "full-count",
    about = "Every pitch. Every play. Every out.",
    long_about = None,
)]
struct Cli {
    /// Load a saved game and resume scoring.
    ///
    /// Accepts a bare name ("game1"), a name with extension ("game1.json"),
    /// a relative path ("./saves/game1.json"), or an absolute path.
    /// Bare names and names with .json are looked up in ~/.full-count/saves/.
    #[arg(short, long, value_name = "SAVE_FILE")]
    load: Option<PathBuf>,
}

/// Resolve a user-supplied save path:
///   1. As-is if it already exists.
///   2. Inside ~/.full-count/saves/.
///   3. Inside ~/.full-count/saves/ with .json appended (bare name).
fn resolve_save_path(arg: &Path) -> PathBuf {
    if arg.exists() {
        return arg.to_path_buf();
    }
    let saves = persist::saves_dir();
    let in_saves = saves.join(arg);
    if in_saves.exists() {
        return in_saves;
    }
    if arg.extension().is_none() {
        let with_ext = saves.join(arg).with_extension("json");
        if with_ext.exists() {
            return with_ext;
        }
    }
    // Return original — load_game will produce a clear error message.
    arg.to_path_buf()
}

// ── Terminal setup ─────────────────────────────────────────────────────────

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn setup_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original(info);
    }));
}

// ── Entry point ───────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    setup_panic_hook();

    let mut app = App::new();

    if let Some(arg) = cli.load {
        let path = resolve_save_path(&arg);
        match persist::load_game(&path) {
            Ok(game) => {
                app.game = Some(game);
                app.screen = AppScreen::Scoring;
            }
            Err(e) => {
                eprintln!("full-count: could not load '{}': {}", arg.display(), e);
                std::process::exit(1);
            }
        }
    }

    let mut terminal = setup_terminal()?;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(Duration::from_millis(16))? {
            let ev = event::read()?;
            input::handle_event(&mut app, ev);
        }

        if app.should_quit {
            break;
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
