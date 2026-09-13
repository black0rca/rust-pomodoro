mod app;
mod events;
mod theme;
mod ui;

use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use signal_hook::consts::signal::{SIGINT, SIGTERM};

use app::App;
use events::{Event, Events};

/// A polished Catppuccin-themed Pomodoro timer for the terminal.
#[derive(Parser, Debug)]
#[command(name = "pomodoro", version, about)]
pub struct Cli {
    /// Work session length in minutes
    #[arg(long, default_value_t = 25)]
    pub work: u64,

    /// Short break length in minutes
    #[arg(long, default_value_t = 5)]
    pub short: u64,

    /// Long break length in minutes
    #[arg(long, default_value_t = 15)]
    pub long: u64,

    /// Number of focus sessions before a long break
    #[arg(long, default_value_t = 4)]
    pub long_after: u32,

    /// Disable desktop notifications (the terminal bell still rings)
    #[arg(long)]
    pub no_notify: bool,
}

type Backend = CrosstermBackend<Stdout>;

fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_panic_hook();
    setup_signal_handlers();

    let mut terminal = init_terminal()?;
    let _guard = RestoreGuard;

    let events = Events::new();
    let mut app = App::new(&cli);
    let quit = setup_signal_handlers();

    let result = run(&mut terminal, &events, &mut app, &quit);
    drop(_guard);
    result
}

fn run(
    terminal: &mut Terminal<Backend>,
    events: &Events,
    app: &mut App,
    quit: &Arc<AtomicBool>,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if quit.load(Ordering::Relaxed) {
            break;
        }

        match events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key) => match key.code {
                KeyCode::Char('q' | 'Q') => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char(' ') => app.toggle_pause(),
                KeyCode::Char('s' | 'n' | 'S' | 'N') => app.skip(),
                KeyCode::Char('r' | 'R') => app.reset(),
                _ => {}
            },
        }
    }
    Ok(())
}

fn init_terminal() -> Result<Terminal<Backend>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
}

struct RestoreGuard;

impl Drop for RestoreGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));
}

fn setup_signal_handlers() -> Arc<AtomicBool> {
    let quit = Arc::new(AtomicBool::new(false));
    let _ = signal_hook::flag::register(SIGINT, Arc::clone(&quit));
    let _ = signal_hook::flag::register(SIGTERM, Arc::clone(&quit));
    quit
}