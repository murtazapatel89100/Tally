use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};
use rust_decimal::Decimal;
use tally_core::printer::format_amount;

use super::form::{Focus, FormState};
use super::theme::{Theme, NORD};
use super::{App, View};

pub fn draw(f: &mut Frame, app: &mut App) {
    let t = &NORD;
    let area = f.area();
    f.render_widget(Block::default().style(t.base()), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // tabs
            Constraint::Min(0),    // body
            Constraint::Length(1), // footer
        ])
        .split(area);

    draw_title(f, chunks[0], app, t);
    draw_tabs(f, chunks[1], app, t);

    match app.view {
        View::Balances => draw_balances(f, chunks[2], app, t),
        View::Register => draw_register(f, chunks[2], app, t),
        View::Accounts => draw_accounts(f, chunks[2], app, t),
    }

    draw_footer(f, chunks[3], app, t);

    if app.show_help {
        draw_help(f, area, t);
    }

    if let Some(form) = &mut app.form {
        draw_form(f, area, form, t);
    }
}

fn draw_title(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let file = app
        .journal_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("journal");
    let txns = app.journal.transactions.len();
    let net = app.net_worth();
    let net_style = if net.starts_with('-') {
        Style::default().fg(t.negative).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.positive).add_modifier(Modifier::BOLD)
    };

    let spans = vec![
        Span::styled(
            " Tally ",
            Style::default().bg(t.accent).fg(t.bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {file}"),
            Style::default().fg(t.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  ·  {txns} txns"), Style::default().fg(t.subtle)),
        Span::styled(
            format!("  ·  {} accounts", app.acc.all.len()),
            Style::default().fg(t.subtle),
        ),
        Span::styled("  ·  net ", Style::default().fg(t.subtle)),
        Span::styled(net, net_style),
    ];
    f.render_widget(Paragraph::new(Line::from(spans)).style(t.base()), area);
}

fn draw_tabs(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
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
            spans.push(Span::raw(" "));
        }
        if app.view == *view {
            spans.push(Span::styled(
                format!(" {label} "),
                Style::default()
                    .bg(t.selection_bg)
                    .fg(t.selection_fg)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(format!(" {label} "), Style::default().fg(t.subtle)));
        }
    }

    if !app.drill_stack.is_empty()
        && let Some(acc) = &app.reg.account_filter
    {
        spans.push(Span::styled(format!("  ▸ {acc}"), Style::default().fg(t.positive)));
    }

    if app.filtering || !app.filter.is_empty() {
        spans.push(Span::styled(
            "   /",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(app.filter.clone(), Style::default().fg(t.text)));
        if app.filtering {
            spans.push(Span::styled("▏", Style::default().fg(t.accent)));
        }
        if !app.filter.is_empty() {
            spans.push(Span::styled(
                format!("  ({} matches)", app.result_count()),
                Style::default().fg(t.subtle),
            ));
        }
    }

    f.render_widget(Paragraph::new(Line::from(spans)).style(t.base()), area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let pairs: &[(&str, &str)] = if app.filtering {
        &[("Enter", "confirm"), ("Esc", "cancel")]
    } else {
        &[
            ("j/k", "move"),
            ("/", "filter"),
            ("Enter", "drill"),
            ("u", "back"),
            ("n", "new"),
            ("e", "edit"),
            ("Space", "collapse"),
            ("[ ]", "all"),
            ("?", "help"),
            ("q", "quit"),
        ]
    };

    let mut spans = vec![Span::raw(" ")];
    for (i, (k, d)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        spans.push(t.key(k));
        spans.push(Span::raw(" "));
        spans.push(t.desc(d));
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(t.base()), area);
}

fn draw_balances(f: &mut Frame, area: Rect, app: &mut App, t: &Theme) {
    let block = t.block(" Balances ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    app.viewport_height = inner.height;
    app.list_top = inner.y;

    if app.bal.visible.is_empty() {
        draw_empty(f, inner, t, empty_msg(app));
        return;
    }

    let width = inner.width as usize;
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
            let marker = if has_children {
                if app.bal.collapsed.contains(&acc_str) { "▸" } else { "▾" }
            } else {
                "•"
            };
            let name = row.account.0.last().map(String::as_str).unwrap_or("");
            let indent = "  ".repeat(row.depth);
            let left = format!("{indent}{marker} {name}");

            let amt = row.amounts.first().map(format_amount).unwrap_or_default();
            let qty = row.amounts.first().map(|a| a.quantity).unwrap_or(Decimal::ZERO);

            let name_style = if row.depth == 0 {
                Style::default().fg(t.selection_fg).add_modifier(Modifier::BOLD)
            } else if row.depth == 1 {
                Style::default().fg(t.text)
            } else {
                Style::default().fg(t.subtle)
            };
            let mut amt_style = t.amount_style(qty);
            if row.depth == 0 {
                amt_style = amt_style.add_modifier(Modifier::BOLD);
            }

            let used = left.chars().count() + amt.chars().count();
            let pad = width.saturating_sub(used).max(1);
            Line::from(vec![
                Span::styled(left, name_style),
                Span::raw(" ".repeat(pad)),
                Span::styled(amt, amt_style),
            ])
        })
        .map(ListItem::new)
        .collect();

    let list = List::new(items).style(t.base()).highlight_style(t.selection());
    f.render_stateful_widget(list, inner, &mut app.bal.list_state);
}

fn draw_register(f: &mut Frame, area: Rect, app: &mut App, t: &Theme) {
    let title = match &app.reg.account_filter {
        Some(acc) => format!(" Register — {acc} "),
        None => " Register ".to_string(),
    };
    let block = t.block(&title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    app.viewport_height = inner.height.saturating_sub(1);
    app.list_top = inner.y + 1;

    if app.reg.rows.is_empty() {
        draw_empty(f, inner, t, empty_msg(app));
        return;
    }

    let header = Row::new(["Date", "Payee", "Account", "Amount", "Balance"]).style(
        Style::default()
            .fg(t.header)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    );

    let mut group = 0usize;
    let mut last_txn: Option<usize> = None;
    let rows: Vec<Row> = app
        .reg
        .rows
        .iter()
        .map(|row| {
            let same = last_txn == Some(row.txn_idx);
            if !same {
                if last_txn.is_some() {
                    group += 1;
                }
                last_txn = Some(row.txn_idx);
            }

            let date = if same { String::new() } else { row.date.to_string() };
            let payee = if same { String::new() } else { trunc(&row.payee, 20) };
            let account = trunc(&row.account.as_str(), 22);
            let amt = row.amount.as_ref().map(format_amount).unwrap_or_default();
            let amt_qty = row.amount.as_ref().map(|a| a.quantity).unwrap_or(Decimal::ZERO);
            let bal_amt = row.running.first();
            let bal = bal_amt.map(format_amount).unwrap_or_else(|| "0".into());
            let bal_neg = bal_amt.map(|a| a.quantity < Decimal::ZERO).unwrap_or(false);
            let bal_style = Style::default().fg(if bal_neg { t.negative } else { t.running });

            let row_bg = if group % 2 == 1 {
                Style::default().bg(t.surface)
            } else {
                t.base()
            };

            Row::new([
                Cell::from(date).style(Style::default().fg(t.subtle)),
                Cell::from(payee).style(Style::default().fg(t.text)),
                Cell::from(account).style(Style::default().fg(t.subtle)),
                Cell::from(Text::from(amt).right_aligned()).style(t.amount_style(amt_qty)),
                Cell::from(Text::from(bal).right_aligned()).style(bal_style),
            ])
            .style(row_bg)
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
        .style(t.base())
        .row_highlight_style(t.selection());
    f.render_stateful_widget(table, inner, &mut app.reg.table_state);
}

fn draw_accounts(f: &mut Frame, area: Rect, app: &mut App, t: &Theme) {
    let block = t.block(" Accounts ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    app.viewport_height = inner.height;
    app.list_top = inner.y;

    if app.acc.filtered.is_empty() {
        draw_empty(f, inner, t, empty_msg(app));
        return;
    }

    let filter = app.filter.clone();
    let items: Vec<ListItem> = app
        .acc
        .filtered
        .iter()
        .map(|&i| {
            let text = app.acc.all[i].as_str();
            ListItem::new(Line::from(highlight(&text, &filter, t)))
        })
        .collect();

    let list = List::new(items).style(t.base()).highlight_style(t.selection());
    f.render_stateful_widget(list, inner, &mut app.acc.list_state);
}

fn draw_form(f: &mut Frame, area: Rect, form: &mut FormState, t: &Theme) {
    let popup = form_rect(area);
    f.render_widget(Clear, popup);

    let title = match &form.mode {
        super::form::FormMode::New => " New Transaction ",
        super::form::FormMode::Edit { .. } => " Edit Transaction ",
    };
    let outer = t.focus_block(title);
    let inner = outer.inner(popup);
    f.render_widget(outer, popup);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // date + status
            Constraint::Length(3), // payee
            Constraint::Length(1), // postings label
            Constraint::Min(2),    // posting rows
            Constraint::Length(1), // balance bar
            Constraint::Length(1), // error
            Constraint::Length(1), // hints
        ])
        .margin(1)
        .split(inner);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(0)])
        .split(rows[0]);

    render_input(f, top_cols[0], &form.date, form.focus == Focus::Date, "Date", t);

    let status_text = match form.status {
        tally_core::model::Status::Uncleared => "  Uncleared",
        tally_core::model::Status::Cleared => "* Cleared",
        tally_core::model::Status::Pending => "! Pending",
    };
    let status_focused = form.focus == Focus::Status;
    let (sb, st) = if status_focused {
        (t.border_focus, t.accent)
    } else {
        (t.border, t.subtle)
    };
    f.render_widget(
        Paragraph::new(Span::styled(status_text, Style::default().fg(t.text)))
            .style(t.base())
            .block(
                Block::default()
                    .title(Span::styled(" Status ", Style::default().fg(st)))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(sb)),
            ),
        top_cols[1],
    );

    render_input(f, rows[1], &form.payee, form.focus == Focus::Payee, "Payee", t);

    f.render_widget(
        Paragraph::new(" Postings").style(Style::default().fg(t.header).bg(t.bg)),
        rows[2],
    );

    let posting_area = rows[3];
    let n = form.postings.len().min(6);
    let constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(3)).collect();
    if !constraints.is_empty() {
        let posting_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(posting_area);

        let mut ac_popup: Option<Rect> = None;
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
            render_input(f, cols[0], &form.postings[i].account, acc_focused, "Account", t);
            render_input(f, cols[1], &form.postings[i].amount, amt_focused, "Amount", t);

            if acc_focused && form.completion_open && !form.completions.is_empty() {
                ac_popup = Some(cols[0]);
            }
        }
        if let Some(acc_area) = ac_popup {
            draw_autocomplete(f, acc_area, form, t);
        }
    }

    // Prominent balance bar.
    let bar_style = if form.balance_ok {
        Style::default().bg(t.positive).fg(t.bg).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(t.negative).fg(t.bg).add_modifier(Modifier::BOLD)
    };
    f.render_widget(
        Paragraph::new(format!("  {}", form.balance_note)).style(bar_style),
        rows[4],
    );

    if let Some(ref err) = form.error {
        f.render_widget(
            Paragraph::new(format!("  ⚠ {err}"))
                .style(Style::default().fg(t.negative).bg(t.bg)),
            rows[5],
        );
    }

    let hints = Line::from(vec![
        Span::raw("  "),
        t.key("Ctrl+S"),
        Span::raw(" "),
        t.desc("save"),
        Span::raw("   "),
        t.key("Esc"),
        Span::raw(" "),
        t.desc("cancel"),
        Span::raw("   "),
        t.key("Tab"),
        Span::raw(" "),
        t.desc("next"),
        Span::raw("   "),
        t.key("Ctrl+N"),
        Span::raw(" "),
        t.desc("posting"),
        Span::raw("   "),
        t.key("Ctrl+D"),
        Span::raw(" "),
        t.desc("delete"),
    ]);
    f.render_widget(Paragraph::new(hints).style(t.base()), rows[6]);
}

fn draw_autocomplete(f: &mut Frame, acc_area: Rect, form: &FormState, t: &Theme) {
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
                t.selection()
            } else {
                Style::default().fg(t.text)
            };
            ListItem::new(Span::styled(format!(" {s}"), style))
        })
        .collect();

    f.render_widget(
        List::new(items).style(t.base()).block(
            Block::default()
                .title(Span::styled(" accounts ", Style::default().fg(t.accent)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.border_focus)),
        ),
        popup,
    );
}

fn render_input(
    f: &mut Frame,
    area: Rect,
    input: &super::form::TextInput,
    focused: bool,
    label: &str,
    t: &Theme,
) {
    let (bstyle, tstyle) = if focused {
        (t.border_focus, t.accent)
    } else {
        (t.border, t.subtle)
    };
    let block = Block::default()
        .title(Span::styled(format!(" {label} "), Style::default().fg(tstyle)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(bstyle))
        .style(t.base());
    let inner = block.inner(area);
    f.render_widget(block, area);

    if focused {
        let text = &input.text;
        let cur = input.cursor.min(text.len());
        let before = &text[..cur];
        let at: String = text[cur..]
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_else(|| " ".into());
        let after_start = cur + at.len();
        let after = if after_start <= text.len() { &text[after_start..] } else { "" };

        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(before.to_string(), Style::default().fg(t.text)),
                Span::styled(at, Style::default().bg(t.accent).fg(t.bg)),
                Span::styled(after.to_string(), Style::default().fg(t.text)),
            ]))
            .style(t.base()),
            inner,
        );
    } else {
        f.render_widget(
            Paragraph::new(Span::styled(input.text.clone(), Style::default().fg(t.text)))
                .style(t.base()),
            inner,
        );
    }
}

fn draw_help(f: &mut Frame, area: Rect, t: &Theme) {
    let head = |s: &str| Line::from(Span::styled(s.to_string(), Style::default().fg(t.accent).add_modifier(Modifier::BOLD)));
    let row = |k: &str, d: &str| {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{k:<12}"), Style::default().fg(t.running).add_modifier(Modifier::BOLD)),
            Span::styled(d.to_string(), Style::default().fg(t.text)),
        ])
    };

    let lines = vec![
        Line::from(""),
        head("  Navigation"),
        row("j / k", "scroll down / up"),
        row("g / G", "top / bottom"),
        row("PgUp/PgDn", "page up / down"),
        row("Ctrl+U/D", "half-page up / down"),
        row("mouse", "wheel scroll · click to select"),
        Line::from(""),
        head("  Views"),
        row("1/b 2/r 3/a", "Balances / Register / Accounts"),
        row("Tab", "cycle views"),
        Line::from(""),
        head("  Actions"),
        row("/", "filter (shows match count)"),
        row("Enter", "drill into register"),
        row("u / Esc", "go back"),
        row("Space / o", "toggle collapse"),
        row("[ / ]", "collapse all / expand all"),
        row("n", "new transaction"),
        row("e", "edit transaction (Register)"),
        Line::from(""),
        head("  Entry form"),
        row("Ctrl+S", "save"),
        row("Tab/Shift+Tab", "next / prev field"),
        row("Ctrl+N/Ctrl+D", "add / delete posting"),
        row("Space", "cycle status"),
        Line::from(""),
        row("q", "quit"),
        Line::from(""),
        Line::from(Span::styled("  press any key to close", Style::default().fg(t.subtle))),
    ];

    let w = 56u16;
    let h = (lines.len() + 2) as u16;
    let popup = centered_rect(w, h, area);
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines).style(t.base()).block(
            Block::default()
                .title(Span::styled(" Help ", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(t.border_focus)),
        ),
        popup,
    );
}

fn draw_empty(f: &mut Frame, area: Rect, t: &Theme, msg: String) {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    f.render_widget(
        Paragraph::new(msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(t.subtle).bg(t.bg)),
        v[1],
    );
}

fn empty_msg(app: &App) -> String {
    if app.filter.is_empty() {
        "Nothing to show".to_string()
    } else {
        "No matches — press Esc to clear the filter".to_string()
    }
}

fn highlight(text: &str, filter: &str, t: &Theme) -> Vec<Span<'static>> {
    let plain = |s: &str| Span::styled(s.to_string(), Style::default().fg(t.text));
    if filter.is_empty() {
        return vec![plain(text)];
    }
    let lower = text.to_lowercase();
    let needle = filter.to_lowercase();
    if let Some(pos) = lower.find(&needle) {
        let end = pos + needle.len();
        if text.is_char_boundary(pos) && text.is_char_boundary(end) {
            return vec![
                plain(&text[..pos]),
                Span::styled(
                    text[pos..end].to_string(),
                    Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
                ),
                plain(&text[end..]),
            ];
        }
    }
    vec![plain(text)]
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
