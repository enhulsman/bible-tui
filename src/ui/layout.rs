use ratatui::layout::{Constraint, Layout, Rect};

pub const NAV_PANEL_MIN_WIDTH: u16 = 60;
pub const NAV_PANEL_WIDTH: u16 = 22;

pub struct AppLayout {
    pub nav_panel: Option<Rect>,
    pub reading_pane: Rect,
    pub status_bar: Rect,
}

pub fn compute_layout(area: Rect, show_nav: bool) -> AppLayout {
    // Split: main area + status bar (1 row)
    let vertical = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);

    let main_area = vertical[0];
    let status_bar = vertical[1];

    if show_nav && area.width >= NAV_PANEL_MIN_WIDTH {
        let horizontal = Layout::horizontal([
            Constraint::Length(NAV_PANEL_WIDTH),
            Constraint::Min(1),
        ])
        .split(main_area);

        AppLayout {
            nav_panel: Some(horizontal[0]),
            reading_pane: horizontal[1],
            status_bar,
        }
    } else {
        AppLayout {
            nav_panel: None,
            reading_pane: main_area,
            status_bar,
        }
    }
}
