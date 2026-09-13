use std::io::{self, Write};
use std::time::Duration;

use notify_rust::Notification;

use crate::events::TICK_RATE;
use crate::Cli;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn icon(self) -> &'static str {
        match self {
            Self::Focus => "⚡",
            Self::ShortBreak => "☕",
            Self::LongBreak => "🌴",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Focus => "FOCUS SESSION",
            Self::ShortBreak => "SHORT BREAK",
            Self::LongBreak => "LONG BREAK",
        }
    }
}

pub struct App {
    pub phase: Phase,
    pub sessions_done: u32,
    pub cycle: u32,
    pub cycle_done: u32,
    pub remaining: Duration,
    pub total: Duration,
    pub paused: bool,
    pub long_after: u32,
    work: Duration,
    short: Duration,
    long: Duration,
    notify: bool,
}

impl App {
    pub fn new(cli: &Cli) -> Self {
        let work = Duration::from_secs(cli.work * 60);
        let short = Duration::from_secs(cli.short * 60);
        let long = Duration::from_secs(cli.long * 60);
        Self {
            phase: Phase::Focus,
            sessions_done: 0,
            cycle: 0,
            cycle_done: 0,
            remaining: work,
            total: work,
            paused: false,
            long_after: cli.long_after.max(1),
            work,
            short,
            long,
            notify: !cli.no_notify,
        }
    }

    pub fn badge(&self) -> String {
        match self.phase {
            Phase::Focus => format!(
                "{} {} #{}",
                self.phase.icon(),
                self.phase.name(),
                self.cycle_done + 1
            ),
            _ => format!("{} {}", self.phase.icon(), self.phase.name()),
        }
    }

    pub fn time_left(&self) -> (u64, u64) {
        let secs = self.remaining.as_millis().div_ceil(1000) as u64;
        (secs / 60, secs % 60)
    }

    pub fn progress(&self) -> f64 {
        if self.total.is_zero() {
            0.0
        } else {
            1.0 - self.remaining.as_secs_f64() / self.total.as_secs_f64()
        }
    }

    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
        self.remaining = self.remaining.saturating_sub(TICK_RATE);
        if self.remaining.is_zero() {
            self.complete_phase();
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn skip(&mut self) {
        match self.phase {
            Phase::Focus => {
                self.phase = if (self.cycle_done + 1).is_multiple_of(self.long_after) {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                };
            }
            _ => self.phase = Phase::Focus,
        }
        self.total = self.duration_for(self.phase);
        self.remaining = self.total;
        self.paused = false;
    }

    pub fn reset(&mut self) {
        self.remaining = self.duration_for(self.phase);
        self.paused = false;
    }

    fn complete_phase(&mut self) {
        self.ring_bell();
        match self.phase {
            Phase::Focus => {
                self.sessions_done += 1;
                self.cycle_done += 1;
                let long_break = self.cycle_done.is_multiple_of(self.long_after);
                self.phase = if long_break {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                };
                if long_break {
                    self.notify("Pomodoro complete!", "Time for a long break — you earned it.");
                } else {
                    self.notify("Pomodoro complete!", "Time for a short break.");
                }
            }
            Phase::ShortBreak => {
                self.phase = Phase::Focus;
                self.notify("Break over", "Back to focus. You've got this.");
            }
            Phase::LongBreak => {
                self.cycle += 1;
                self.cycle_done = 0;
                self.phase = Phase::Focus;
                self.notify("Long break over", "New cycle. Back to focus.");
            }
        }
        self.total = self.duration_for(self.phase);
        self.remaining = self.total;
        self.paused = false;
    }

    fn duration_for(&self, phase: Phase) -> Duration {
        match phase {
            Phase::Focus => self.work,
            Phase::ShortBreak => self.short,
            Phase::LongBreak => self.long,
        }
    }

    fn notify(&self, summary: &str, body: &str) {
        if self.notify {
            let _ = Notification::new().summary(summary).body(body).show();
        }
    }

    fn ring_bell(&self) {
        let mut out = io::stdout();
        let _ = write!(out, "\x07");
        let _ = out.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cli;
    use clap::Parser;

    fn app() -> App {
        App::new(&Cli::parse_from(["pomodoro", "--no-notify"]))
    }

    fn finish_phase(a: &mut App) {
        a.remaining = TICK_RATE;
        a.tick();
    }

    #[test]
    fn ticks_down_incrementally() {
        let mut a = app();
        a.tick();
        assert_eq!(a.remaining, Duration::from_secs(25 * 60) - TICK_RATE);
        assert_eq!(a.phase, Phase::Focus);
    }

    #[test]
    fn paused_does_not_tick() {
        let mut a = app();
        a.paused = true;
        let before = a.remaining;
        a.tick();
        assert_eq!(a.remaining, before);
    }

    #[test]
    fn completion_advances_to_short_break() {
        let mut a = app();
        finish_phase(&mut a);
        assert_eq!(a.phase, Phase::ShortBreak);
        assert_eq!(a.cycle_done, 1);
        assert_eq!(a.sessions_done, 1);
        assert_eq!(a.remaining, a.short);
    }

    #[test]
    fn fourth_completion_advances_to_long_break() {
        let mut a = app();
        for _ in 0..3 {
            finish_phase(&mut a);
            finish_phase(&mut a);
        }
        finish_phase(&mut a);
        assert_eq!(a.phase, Phase::LongBreak);
        assert_eq!(a.cycle_done, 4);
    }

    #[test]
    fn long_break_resets_cycle() {
        let mut a = app();
        for _ in 0..4 {
            finish_phase(&mut a);
            finish_phase(&mut a);
        }
        assert_eq!(a.phase, Phase::Focus);
        assert_eq!(a.cycle_done, 0);
        assert_eq!(a.cycle, 1);
        assert_eq!(a.sessions_done, 4);
    }

    #[test]
    fn skip_moves_to_next_phase_without_crediting() {
        let mut a = app();
        a.skip();
        assert_eq!(a.phase, Phase::ShortBreak);
        assert_eq!(a.sessions_done, 0);
        a.skip();
        assert_eq!(a.phase, Phase::Focus);
    }

    #[test]
    fn reset_restores_full_duration() {
        let mut a = app();
        a.remaining = Duration::from_secs(60);
        a.paused = true;
        a.reset();
        assert_eq!(a.remaining, a.work);
        assert!(!a.paused);
    }

    #[test]
    fn badge_shows_focus_session_number() {
        let mut a = app();
        assert_eq!(a.badge(), "⚡ FOCUS SESSION #1");
        finish_phase(&mut a);
        assert_eq!(a.badge(), "☕ SHORT BREAK");
        a.skip();
        assert_eq!(a.badge(), "⚡ FOCUS SESSION #2");
    }
}