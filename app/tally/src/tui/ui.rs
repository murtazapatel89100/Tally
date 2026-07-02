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
use super::form::{Focus, FormState};

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

    if let Some(form) = &mut app.form {
        draw_form(f, area, form);
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
        spans.push(Span::styled("   /", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(app.filter.clone(), Style::default().fg(Color::White)));
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
        "  j/k: move   /: filter   Enter: drill   u/Esc: back   n: new   e: edit   Space: collapse   ?: help   q: quit"
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
            let negative = row.amounts.first().map(|a| a.quantity < Decimal::ZERO).unwrap_or(false);
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

    let rows: Vec<Row> = app.reg.rows.iter().map(|row| {
        let date = row.date.to_string();
        let payee = trunc(&row.payee, 20);
        let account = trunc(&row.account.as_str(), 22);
        let amt = row.amount.as_ref().map(format_amount).unwrap_or_default();
        let bal = row.running.first().map(format_amount).unwrap_or_else(|| "0".into());
        let negative = row.amount.as_ref().map(|a| a.quantity < Decimal::ZERO).unwrap_or(false);
        let amt_style = Style::default().fg(if negative { Color::Red } else { Color::Green });
        Row::new([
            Cell::from(date),
            Cell::from(payee),
            Cell::from(account),
            Cell::from(amt).style(amt_style),
            Cell::from(bal).style(Style::default().fg(Color::Cyan)),
        ])
    }).collect();

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

    let items: Vec<ListItem> = app.acc.filtered.iter()
        .map(|&i| ListItem::new(app.acc.all[i].as_str()))
        .collect();
    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, inner, &mut app.acc.list_state);
}

fn draw_form(f: &mut Frame, area: Rect, form: &mut FormState) {
    let popup = form_rect(area);
    f.render_widget(Clear, popup);

    let title = match &form.mode {
        super::form::FormMode::New => " New Transaction ",
        super::form::FormMode::Edit { .. } => " Edit Transaction ",
    };

    let outer = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = outer.inner(popup);
    f.render_widget(outer, popup);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // date + status row
            Constraint::Length(3),  // payee row
            Constraint::Length(1),  // "Postings" label
            Constraint::Min(2),     // posting rows
            Constraint::Length(1),  // balance line
            Constraint::Length(1),  // error line
            Constraint::Length(1),  // footer hints
        ])
        .margin(1)
        .split(inner);

    // Row 1: Date + Status
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(0)])
        .split(rows[0]);

    render_input(f, top_cols[0], &form.date, form.focus == Focus::Date, "Date");

    let status_text = match form.status {
        tally_core::model::Status::Uncleared => "  Uncleared",
        tally_core::model::Status::Cleared => "* Cleared",
        tally_core::model::Status::Pending => "! Pending",
    };
    let status_style = if form.focus == Focus::Status {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    f.render_widget(
        Paragraph::new(status_text)
            .block(Block::default().title(" Status ").borders(Borders::ALL).border_style(status_style)),
        top_cols[1],
    );

    // Row 2: Payee
    render_input(f, rows[1], &form.payee, form.focus == Focus::Payee, "Payee");

    // Postings label
    f.render_widget(
        Paragraph::new(" Postings ─────────────────────────────────────")
            .style(Style::default().fg(Color::DarkGray)),
        rows[2],
    );

    // Posting rows
    let posting_area = rows[3];
    let n = form.postings.len().min(6);
    let constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(3)).collect();
    if !constraints.is_empty() {
        let posting_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(posting_area);

        let mut ac_popup: Option<(Rect, usize)> = None;

        for (i, row_area) in posting_rows.iter().enumerate() {
            if i >= form.postings.len() {
                break;
            }
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(16)])
                .split(*row_area);

            let acc_focused = form.focus == Focus::PostingAccount(i);
            let amt_focused = form.focus == Focus::PostingAmount(i);

            render_input(f, cols[0], &form.postings[i].account, acc_focused, "Account");
            render_input(f, cols[1], &form.postings[i].amount, amt_focused, "Amount");

            if acc_focused && form.completion_open && !form.completions.is_empty() {
                ac_popup = Some((cols[0], i));
            }
        }

        if let Some((acc_area, _)) = ac_popup {
            draw_autocomplete(f, acc_area, form);
        }
    }

    // Balance line
    let bal_style = if form.balance_ok {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };
    f.render_widget(
        Paragraph::new(format!("  {}", form.balance_note)).style(bal_style),
        rows[4],
    );

    // Error line
    if let Some(ref err) = form.error {
        f.render_widget(
            Paragraph::new(format!("  ⚠ {err}")).style(Style::default().fg(Color::Red)),
            rows[5],
        );
    }

    // Footer hints
    f.render_widget(
        Paragraph::new("  Ctrl+S save  Esc cancel  Tab/Shift+Tab next/prev  Ctrl+N posting  Ctrl+D delete  Space toggle status")
            .style(Style::default().fg(Color::DarkGray)),
        rows[6],
    );
}

fn draw_autocomplete(f: &mut Frame, acc_area: Rect, form: &FormState) {
    let n = form.completions.len().min(8) as u16;
    let popup = Rect {
        x: acc_area.x,
        y: (acc_area.y + acc_area.height).min(f.area().height.saturating_sub(n + 2)),
        width: acc_area.width,
        height: n + 2,
    };

    f.render_widget(Clear, popup);
    let items: Vec<ListItem> = form
        .completions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == form.completion_sel {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(format!(" {s}"), style))
        })
        .collect();

    f.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        popup,
    );
}

fn render_input(f: &mut Frame, area: Rect, input: &super::form::TextInput, focused: bool, label: &str) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(Span::styled(format!(" {label} "), border_style))
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if focused {
        let text = &input.text;
        let cur = input.cursor;
        let before = &text[..cur];
        let at: String = text[cur..].chars().next().map(|c| c.to_string()).unwrap_or_else(|| " ".into());
        let after_start = cur + at.len();
        let after = if after_start <= text.len() { &text[after_start..] } else { "" };

        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw(before),
                Span::styled(at, Style::default().bg(Color::White).fg(Color::Black)),
                Span::raw(after),
            ])),
            inner,
        );
    } else {
        f.render_widget(Paragraph::new(input.text.as_str()), inner);
    }
}

fn draw_help(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("  Navigation", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  j / ↓       scroll down"),
        Line::from("  k / ↑       scroll up"),
        Line::from("  g / G       top / bottom"),
        Line::from(""),
        Line::from(vec![Span::styled("  Views", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  1/b  2/r  3/a  Tab   switch views"),
        Line::from(""),
        Line::from(vec![Span::styled("  Actions", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  /           filter"),
        Line::from("  Enter       drill into register"),
        Line::from("  u / Esc     go back"),
        Line::from("  Space / o   toggle collapse"),
        Line::from("  n           new transaction"),
        Line::from("  e           edit transaction (Register)"),
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

fn form_rect(area: Rect) -> Rect {
    let w = area.width.min(82);
    let h = area.height.min(34);
    centered_rect(w, h, area)
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
