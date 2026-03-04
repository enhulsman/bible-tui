use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const HELP_LINES: &[(&str, &str)] = &[
    ("READING", ""),
    ("j / k / ↑ / ↓", "Scroll up/down"),
    ("f / b / PgDn / PgUp", "Page scroll"),
    ("g / G", "Top / Bottom"),
    ("Space / Backspace", "Next / Previous chapter"),
    ("/", "Search"),
    ("n / N", "Next / Prev search result"),
    (":", "Command mode"),
    ("B", "Bookmark current verse"),
    ("?", "Toggle this help"),
    ("q / Ctrl+C", "Quit"),
    ("", ""),
    ("NAVIGATION (Tab)", ""),
    ("j / k / ↑ / ↓", "Move through list"),
    ("h / l / ← / →", "Switch books ↔ chapters"),
    ("Enter", "Go to selection"),
    ("Esc / Tab", "Close panel"),
    ("", ""),
    ("SEARCH (/)", ""),
    ("Type to search", "Min 2 characters"),
    ("↑ / ↓", "Select result"),
    ("Enter", "Go to result"),
    ("Esc", "Close search"),
    ("", ""),
    ("COMMANDS (:)", ""),
    (":q", "Quit"),
    (":goto <ref>", "Go to reference (e.g. John 3:16)"),
    (":t <name>", "Switch translation (kjv/web/sv)"),
];

pub struct HelpOverlay;

impl HelpOverlay {
    pub fn render(frame: &mut Frame, area: Rect) {
        let height = (HELP_LINES.len() as u16 + 4).min(area.height.saturating_sub(2));
        let width = 60u16.min(area.width.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let popup = Rect::new(x, y, width, height);

        frame.render_widget(Clear, popup);

        let lines: Vec<Line> = HELP_LINES
            .iter()
            .map(|(key, desc)| {
                if desc.is_empty() && key.is_empty() {
                    Line::from("")
                } else if desc.is_empty() {
                    // Section header
                    Line::from(Span::styled(
                        key.to_string(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(vec![
                        Span::styled(
                            format!("{:<26}", key),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::styled(desc.to_string(), Style::default().fg(Color::White)),
                    ])
                }
            })
            .collect();

        let widget = Paragraph::new(lines).block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(widget, popup);
    }
}
