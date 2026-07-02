use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};
use rust_decimal::Decimal;
use tally_core::printer::format_amount;

use super::{App, View};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    draw_header(f, chunks[0], app);

    match app.view {
        View::Balances => draw_balances(f, chunks[1], app),
        View::Register => draw_register(f, chunks[1], app),
        View::Accounts => draw_accounts(f, chunks[1], app),
    }

    draw_footer(f, chunks[2], app);

    if app.show_help {
        draw_help(f, area);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let active = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let inactive = Style::default().fg(Color::DarkGray);
    let sep = Span::styled("  │  ", inactive);

    let mut spans = vec![Span::raw(" ")];

    for (i, (label, view)) in [
        ("1 Balances", View::Balances),
        ("2 Register", View::Register),
        ("3 Accounts", View::Accounts),
    ]
    .iter()
    .enumerate()
    {
        if i > 0 {
            spans.push(sep.clone());
        }
        let style = if app.view == *view { active } else { inactive };
        spans.push(Span::styled(*label, style));
    }

    if !app.drill_stack.is_empty() {
        if let Some(ref acc) = app.reg.account_filter {
            spans.push(Span::styled(
                format!("  ▶ {acc}"),
                Style::default().fg(Color::Green),
            ));
        }
    }

    if app.filtering || !app.filter.is_empty() {
        spans.push(Span::styled(
            "   /",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            app.filter.clone(),
            Style::default().fg(Color::White),
        ));
        if app.filtering {
            spans.push(Span::styled("█", Style::default().fg(Color::White)));
        }
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let text = if app.filtering {
        "  Enter: confirm   Esc: cancel"
    } else {
        "  j/k: move   g/G: top/bot   /: filter   Enter: drill   u/Esc: back   Space: collapse   ?: help   q: quit"
    };
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn draw_balances(f: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title(" Balance ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let amt_width = app
        .bal
        .report
        .rows
        .iter()
        .flat_map(|r| r.amounts.iter())
        .map(|a| format_amount(a).len())
        .max()
        .unwrap_or(12)
        .max(12);

    let items: Vec<ListItem> = app
        .bal
        .visible
        .iter()
        .map(|&ri| {
            let row = &app.bal.report.rows[ri];
            let acc_str = row.account.as_str();

            let has_children = app.bal.report.rows.iter().any(|r| {
                let s = r.account.as_str();
                s != acc_str && s.starts_with(&format!("{acc_str}:"))
            });
            let indicator = if has_children {
                if app.bal.collapsed.contains(&acc_str) { "▶ " } else { "▼ " }
            } else {
                "  "
            };

            let display = row.account.0.last().map(String::as_str).unwrap_or("");
            let indent = "  ".repeat(row.depth);
            let amt = row.amounts.first().map(format_amount).unwrap_or_default();
            let negative = row
                .amounts
                .first()
                .map(|a| a.quantity < Decimal::ZERO)
                .unwrap_or(false);
            let amt_style = Style::default().fg(if negative { Color::Red } else { Color::Green });

            ListItem::new(Line::from(vec![
                Span::raw(indicator),
                Span::styled(format!("{:>w$}", amt, w = amt_width), amt_style),
                Span::raw(format!("  {indent}{display}")),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, inner, &mut app.bal.list_state);
}

fn draw_register(f: &mut Frame, area: Rect, app: &mut App) {
    let title = match &app.reg.account_filter {
        Some(acc) => format!(" Register — {acc} "),
        None => " Register ".to_string(),
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let header = Row::new(["Date", "Payee", "Account", "Amount", "Balance"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let rows: Vec<Row> = app
        .reg
        .rows
        .iter()
        .map(|row| {
            let date = row.date.to_string();
            let payee = trunc(&row.payee, 20);
            let account = trunc(&row.account.as_str(), 22);
            let amt = row.amount.as_ref().map(format_amount).unwrap_or_default();
            let bal = row
                .running
                .first()
                .map(format_amount)
                .unwrap_or_else(|| "0".to_string());

            let negative = row
                .amount
                .as_ref()
                .map(|a| a.quantity < Decimal::ZERO)
                .unwrap_or(false);
            let amt_style =
                Style::default().fg(if negative { Color::Red } else { Color::Green });

            Row::new([
                Cell::from(date),
                Cell::from(payee),
                Cell::from(account),
                Cell::from(amt).style(amt_style),
                Cell::from(bal).style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Length(22),
        Constraint::Length(13),
        Constraint::Length(13),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(table, inner, &mut app.reg.table_state);
}

fn draw_accounts(f: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title(" Accounts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = app
        .acc
        .filtered
        .iter()
        .map(|&i| ListItem::new(app.acc.all[i].as_str()))
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, inner, &mut app.acc.list_state);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("  Navigation", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  j / ↓       scroll down"),
        Line::from("  k / ↑       scroll up"),
        Line::from("  g           go to top"),
        Line::from("  G           go to bottom"),
        Line::from(""),
        Line::from(vec![Span::styled("  Views", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  1 / b       Balances"),
        Line::from("  2 / r       Register"),
        Line::from("  3 / a       Accounts"),
        Line::from("  Tab         cycle views"),
        Line::from(""),
        Line::from(vec![Span::styled("  Actions", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  /           filter"),
        Line::from("  Enter       drill into register"),
        Line::from("  u / Esc     go back / clear filter"),
        Line::from("  Space / o   toggle collapse (Balances)"),
        Line::from(""),
        Line::from("  q           quit"),
        Line::from(""),
        Line::from(Span::styled("  press any key to close", Style::default().fg(Color::DarkGray))),
    ];

    let w = 44u16;
    let h = (lines.len() + 2) as u16;
    let popup = centered_rect(w, h, area);

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        ),
        popup,
    );
}

fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(w) / 2,
        y: area.y + area.height.saturating_sub(h) / 2,
        width: w.min(area.width),
        height: h.min(area.height),
    }
}

fn trunc(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
