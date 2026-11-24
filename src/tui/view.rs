//! Column rendering. The left columns behave like a file explorer: each one
//! lists the children of the selection to its left, and the rightmost pane is a
//! scrolling preview of the current selection.

use std::iter;

use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::app::App;
use super::tree::DetailLine;

const TITLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
const SUBTITLE: Style = Style::new().fg(Color::Yellow);
const HEADING: Style = Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD);
const BULLET: Style = Style::new().fg(Color::Green);
const MUTED: Style = Style::new().fg(Color::DarkGray);

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.area());

    draw_header(frame, chunks[0], app);
    draw_columns(frame, chunks[1], app);
    draw_footer(frame, chunks[2]);
}

fn draw_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut spans = vec![
        Span::styled(
            " muhan ",
            Style::new()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];
    for (i, label) in app.breadcrumb().iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" / ", MUTED));
        }
        spans.push(Span::styled(
            (*label).to_string(),
            Style::new().fg(Color::White),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_footer(frame: &mut Frame, area: ratatui::layout::Rect) {
    let hints =
        " ← →  columns    ↑ ↓  move / scroll    Enter  drill in    Backspace  up    q  quit ";
    frame.render_widget(Paragraph::new(Line::from(Span::styled(hints, MUTED))), area);
}

fn draw_columns(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let list_columns = app.columns();
    // Leave the lions share of the screen to the reading pane.
    let list_width = (area.width / (list_columns as u16 + 2)).clamp(12, 30);
    let constraints = (0..list_columns)
        .map(|_| Constraint::Length(list_width))
        .chain(iter::once(Constraint::Min(0)))
        .collect::<Vec<_>>();
    let areas = Layout::horizontal(constraints).split(area);

    for d in 0..list_columns {
        draw_list(frame, areas[d], app, d, list_width);
    }
    draw_detail(frame, areas[list_columns], app);
}

fn draw_list(frame: &mut Frame, area: ratatui::layout::Rect, app: &App, d: usize, width: u16) {
    let inner_width = width.saturating_sub(2) as usize;
    let focused = d == app.focus;
    let nodes = app.column_nodes(d);
    let items = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let selected = i == app.cursor[d];
            let marker = if node.children.is_empty() {
                "·"
            } else {
                "▸"
            };
            let label_style = if selected && focused {
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if selected {
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if focused {
                Style::new().fg(Color::White)
            } else {
                Style::new().fg(Color::Gray)
            };
            let hint_style = if selected && focused {
                Style::new().fg(Color::Black).bg(Color::Cyan)
            } else {
                MUTED
            };
            let mut row = wrap_lines(
                &format!("{marker} {}", node.label),
                inner_width,
                label_style,
                " ",
                " ",
            );
            row.extend(wrap_lines(
                &node.hint,
                inner_width,
                hint_style,
                "   ",
                "   ",
            ));
            ListItem::new(Text::from(row))
        })
        .collect::<Vec<_>>();

    let title = if d == 0 {
        app.tree.label.clone()
    } else {
        app.node_at_depth(d - 1).label.clone()
    };
    let border = if focused {
        Style::new().fg(Color::Cyan)
    } else {
        MUTED
    };
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(border)
        .title(format!(" {title} "));
    let list = List::new(items)
        .block(block)
        .highlight_symbol("")
        .highlight_style(Style::new());

    let mut state = ListState::default();
    state.select(Some(app.cursor[d]));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_detail(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let node = app.detail_node();
    let inner_width = area.width.saturating_sub(2) as usize;
    let text = detail_text(&node.detail, inner_width);
    let inner_height = area.height.saturating_sub(2) as usize;
    let max_scroll = text.lines.len().saturating_sub(inner_height);
    let scroll = app.scroll.min(max_scroll).min(u16::MAX as usize) as u16;

    let focused = app.focus == app.columns();
    let border = if focused {
        Style::new().fg(Color::Cyan)
    } else {
        MUTED
    };
    let title = if max_scroll > 0 {
        format!(" {}  {}/{} ", node.label, scroll + 1, max_scroll + 1)
    } else {
        format!(" {} ", node.label)
    };
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(border)
        .title(title);
    let paragraph = Paragraph::new(text).block(block).scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}

fn wrap_lines(
    text: &str,
    width: usize,
    style: Style,
    initial: &str,
    subsequent: &str,
) -> Vec<Line<'static>> {
    let options = textwrap::Options::new(width.max(4))
        .initial_indent(initial)
        .subsequent_indent(subsequent);
    textwrap::fill(text, options)
        .lines()
        .map(|line| Line::from(Span::styled(line.to_string(), style)))
        .collect()
}

fn detail_text(detail: &[DetailLine], width: usize) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    for entry in detail {
        match entry {
            DetailLine::Blank => lines.push(Line::from("")),
            DetailLine::Title(s) => {
                lines.extend(wrap_lines(s, width, TITLE, "", ""));
            }
            DetailLine::Subtitle(s) => {
                lines.extend(wrap_lines(s, width, SUBTITLE, "", ""));
            }
            DetailLine::Heading(s) => {
                lines.extend(wrap_lines(s, width, HEADING, "", ""));
            }
            DetailLine::Text(s) => {
                lines.extend(wrap_lines(s, width, Style::new(), "", ""));
            }
            DetailLine::Bullet(s) => {
                let marker = "• ";
                for (i, mut line) in wrap_lines(s, width, Style::new(), marker, "  ")
                    .into_iter()
                    .enumerate()
                {
                    if i == 0 {
                        if let Some(span) = line.spans.first_mut() {
                            let content = span.content.to_string();
                            if let Some(rest) = content.strip_prefix(marker) {
                                let rest = rest.to_string();
                                span.content = marker.into();
                                span.style = BULLET;
                                line.spans.insert(1, Span::styled(rest, Style::new()));
                            }
                        }
                    }
                    lines.push(line);
                }
            }
        }
    }
    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;
    use crate::data::Portfolio;
    use crate::tui::app::App;

    fn screen(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..height {
            for x in 0..width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn renders_columns_and_wrapped_detail() {
        let mut app = App::new(&Portfolio::load());
        let rendered = screen(&app, 100, 30);
        assert!(rendered.contains("Portfolio / About"));
        assert!(rendered.contains("Experience"));
        assert!(rendered.contains("all live here"));

        app.move_right();
        let rendered = screen(&app, 100, 30);
        assert!(rendered.contains("About / Profile"));
        assert!(rendered.contains("muhanli.work@gmail.com"));
        assert!(rendered.contains("Name, tagline"));
    }

    #[test]
    fn long_role_details_scroll_in_the_pane() {
        let mut app = App::new(&Portfolio::load());
        app.move_down(); // Experience
        app.move_right(); // into the roles column
        app.move_right(); // focus the detail pane

        let first = screen(&app, 100, 30);
        assert!(first.contains("Software Engineer Intern @ Apera"));
        assert!(first.contains("asset team at Apera"));

        for _ in 0..40 {
            app.move_down();
        }
        let scrolled = screen(&app, 100, 30);
        assert_ne!(scrolled, first, "scrolling changes the visible detail");
        assert!(scrolled.contains("Stack"), "later sections become visible");
    }
}
