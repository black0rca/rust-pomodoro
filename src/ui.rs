use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
    Frame,
};

use crate::app::{App, Phase};
use crate::theme;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame
        .buffer_mut()
        .set_style(area, Style::default().bg(theme::BASE));

    if area.width < 42 || area.height < 18 {
        frame.render_widget(
            Paragraph::new("Terminal too small")
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme::OVERLAY2)),
            area,
        );
        return;
    }

    let accent = theme::accent(app.phase);
    let muted = if app.paused { theme::OVERLAY1 } else { theme::OVERLAY0 };

    let layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(17),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ]);
    let [_, card_area, _, footer_area, _] = layout.areas(area);

    let card_layout = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(56),
        Constraint::Fill(1),
    ]);
    let [_, card_area, _] = card_layout.areas(card_area);

    render_card(frame, card_area, app, accent, muted);
    render_footer(frame, footer_area, muted);
}

fn render_card(frame: &mut Frame, area: Rect, app: &App, accent: Color, muted: Color) {
    let title = Line::from(vec![
        Span::styled(app.badge(), Style::default().fg(accent).bold()),
        if app.paused {
            Span::styled(" ⏸", Style::default().fg(muted))
        } else {
            Span::raw("")
        },
    ]);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if app.paused { muted } else { accent }))
        .bg(theme::SURFACE0)
        .title(title)
        .title_alignment(Alignment::Center);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ]);
    let [_, timer, _, bar, _, dots, _, divider, stats, _] = rows.areas(inner);

    let (minutes, seconds) = app.time_left();
    render_timer(
        frame,
        timer,
        &format!("{minutes:02}:{seconds:02}"),
        if app.paused { muted } else { accent },
    );

    let bar_area = Rect {
        x: bar.x + 1,
        y: bar.y,
        width: bar.width.saturating_sub(2),
        height: 1,
    };
    frame.render_widget(
        Bar {
            ratio: app.progress(),
            fg: accent,
            bg: theme::SURFACE1,
        },
        bar_area,
    );

    render_cycle_dots(frame, dots, app, muted);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(divider.width as usize),
            Style::default().fg(theme::OVERLAY0),
        ))),
        divider,
    );

    render_stats(frame, stats, app);
}

const GLYPHS: [[&str; 5]; 11] = [
    ["███", "█ █", "█ █", "█ █", "███"],
    [" █ ", "██ ", " █ ", " █ ", " █ "],
    ["███", "  █", "███", "█  ", "███"],
    ["███", "  █", "███", "  █", "███"],
    ["█ █", "█ █", "███", "  █", "  █"],
    ["███", "█  ", "███", "  █", "███"],
    ["███", "█  ", "███", "█ █", "███"],
    ["███", "  █", "  █", "  █", "  █"],
    ["███", "█ █", "███", "█ █", "███"],
    ["███", "█ █", "███", "  █", "███"],
    ["   ", " █ ", "   ", " █ ", "   "],
];

fn glyph(ch: char) -> &'static [&'static str; 5] {
    match ch {
        '0'..='9' => &GLYPHS[(ch as u8 - b'0') as usize],
        _ => &GLYPHS[10],
    }
}

fn render_timer(frame: &mut Frame, area: Rect, text: &str, color: Color) {
    let mut rows: [String; 5] = std::array::from_fn(|_| String::new());
    for (i, ch) in text.chars().enumerate() {
        if i > 0 {
            for row in rows.iter_mut() {
                row.push(' ');
            }
        }
        for (row, cell) in rows.iter_mut().zip(glyph(ch).iter()) {
            row.push_str(cell);
        }
    }
    let width = rows[0].chars().count() as u16;
    let x = area.x + area.width.saturating_sub(width).saturating_div(2);
    let style = Style::default().fg(color).bold();
    for (row_idx, row) in rows.iter().enumerate() {
        frame
            .buffer_mut()
            .set_string(x, area.y + row_idx as u16, row.as_str(), style);
    }
}

struct Bar {
    ratio: f64,
    fg: Color,
    bg: Color,
}

const SUB_BLOCKS: [&str; 9] = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];

impl Widget for Bar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        if width == 0 {
            return;
        }
        let fill = (self.ratio * width as f64).clamp(0.0, width as f64);
        let whole = fill.floor() as usize;
        let sub = ((fill - whole as f64) * 8.0).round() as usize;
        for i in 0..width {
            let (ch, fg) = if i < whole {
                ("█", self.fg)
            } else if i == whole {
                (SUB_BLOCKS[sub.min(8)], self.fg)
            } else {
                (" ", self.fg)
            };
            buf.set_string(
                area.x + i as u16,
                area.y,
                ch,
                Style::default().fg(fg).bg(self.bg),
            );
        }
    }
}

fn render_cycle_dots(frame: &mut Frame, area: Rect, app: &App, muted: Color) {
    let mut spans = vec![Span::styled("[ ", Style::default().fg(muted))];
    for i in 0..app.long_after {
        let done = i < app.cycle_done;
        let fg = if done { theme::LAVENDER } else { theme::OVERLAY0 };
        spans.push(Span::styled(if done { "● " } else { "○ " }, Style::default().fg(fg)));
    }
    spans.push(Span::styled("]", Style::default().fg(muted)));
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn render_stats(frame: &mut Frame, area: Rect, app: &App) {
    let next = match app.phase {
        Phase::Focus => {
            if (app.cycle_done + 1).is_multiple_of(app.long_after) {
                "Long Break"
            } else {
                "Short Break"
            }
        }
        _ => "Focus Session",
    };
    let text = format!(
        "Pomodoros: {}  ·  Cycle: {}  ·  Next: {}",
        app.sessions_done,
        app.cycle + 1,
        next
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(theme::SUBTEXT0),
        )))
        .alignment(Alignment::Center),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect, muted: Color) {
    let keys = [
        ("Space", "Pause/Resume"),
        ("s", "Skip"),
        ("r", "Reset"),
        ("q", "Quit"),
    ];
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("   ", Style::default().fg(muted)));
        }
        spans.push(Span::styled("[", Style::default().fg(muted)));
        spans.push(Span::styled(
            *key,
            Style::default().fg(theme::LAVENDER).bold(),
        ));
        spans.push(Span::styled("]", Style::default().fg(muted)));
        spans.push(Span::styled(
            format!(" {desc}"),
            Style::default().fg(theme::OVERLAY1),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}