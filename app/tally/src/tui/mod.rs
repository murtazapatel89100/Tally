pub mod form;
pub mod theme;
pub mod ui;

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::miette;
use ratatui::{backend::CrosstermBackend, widgets::{ListState, TableState}, Terminal};
use tally_core::{
    journal::Journal,
    model::Account,
    query::Query,
    report::{self, BalReport, RegRow},
};

use form::{FormState, Focus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Balances,
    Register,
    Accounts,
}

pub struct BalState {
    pub report: BalReport,
    pub visible: Vec<usize>,
    pub list_state: ListState,
    pub collapsed: HashSet<String>,
}

pub struct RegState {
    pub rows: Vec<RegRow>,
    pub table_state: TableState,
    pub account_filter: Option<String>,
}

pub struct AccState {
    pub all: Vec<Account>,
    pub filtered: Vec<usize>,
    pub list_state: ListState,
}

pub struct App {
    pub journal: Journal,
    pub journal_path: PathBuf,
    pub view: View,
    pub filter: String,
    pub filtering: bool,
    pub show_help: bool,
    pub bal: BalState,
    pub reg: RegState,
    pub acc: AccState,
    pub drill_stack: Vec<(View, Option<String>)>,
    pub form: Option<FormState>,
    pub viewport_height: u16,
    pub list_top: u16,
}

fn compute_visible(report: &BalReport, collapsed: &HashSet<String>) -> Vec<usize> {
    report
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            let s = row.account.as_str();
            let parts: Vec<&str> = s.split(':').collect();
            for d in 1..parts.len() {
                if collapsed.contains(&parts[..d].join(":")) {
                    return false;
                }
            }
            true
        })
        .map(|(i, _)| i)
        .collect()
}

impl App {
    pub fn new(journal: Journal, path: PathBuf) -> Self {
        let q = Query::default();
        let bal_report = report::balance(&journal, &q);
        let reg_report = report::register(&journal, &q);
        let accounts: Vec<Account> = journal.accounts.iter().cloned().collect();
        let n = accounts.len();
        let collapsed = HashSet::new();
        let visible = compute_visible(&bal_report, &collapsed);

        let mut bal_ls = ListState::default();
        if !visible.is_empty() {
            bal_ls.select(Some(0));
        }
        let mut reg_ts = TableState::default();
        if !reg_report.rows.is_empty() {
            reg_ts.select(Some(0));
        }
        let mut acc_ls = ListState::default();
        if n > 0 {
            acc_ls.select(Some(0));
        }

        App {
            journal,
            journal_path: path,
            view: View::Balances,
            filter: String::new(),
            filtering: false,
            show_help: false,
            bal: BalState {
                report: bal_report,
                visible,
                list_state: bal_ls,
                collapsed,
            },
            reg: RegState {
                rows: reg_report.rows,
                table_state: reg_ts,
                account_filter: None,
            },
            acc: AccState {
                filtered: (0..n).collect(),
                all: accounts,
                list_state: acc_ls,
            },
            drill_stack: Vec::new(),
            form: None,
            viewport_height: 0,
            list_top: 0,
        }
    }

    pub fn reload(&mut self, journal: Journal) {
        self.journal = journal;
        let q = Query::default();
        let bal_report = report::balance(&self.journal, &q);
        let reg_report = report::register(&self.journal, &q);
        let accounts: Vec<Account> = self.journal.accounts.iter().cloned().collect();
        let n = accounts.len();
        let visible = compute_visible(&bal_report, &self.bal.collapsed);

        self.bal.report = bal_report;
        self.bal.visible = visible;
        self.bal.list_state.select(Some(0));

        self.reg.rows = reg_report.rows;
        self.reg.table_state.select(Some(0));
        self.reg.account_filter = None;

        self.acc.all = accounts;
        self.acc.filtered = (0..n).collect();
        self.acc.list_state.select(Some(0));

        self.drill_stack.clear();
        self.filter.clear();
        self.filtering = false;
    }

    pub fn open_new_form(&mut self) {
        self.form = Some(FormState::new_transaction());
        if let Some(f) = &mut self.form {
            f.update_completions(&self.acc.all);
        }
    }

    pub fn open_edit_form(&mut self) {
        if self.view != View::Register {
            return;
        }
        let sel = match self.reg.table_state.selected() {
            Some(i) => i,
            None => return,
        };
        let row = match self.reg.rows.get(sel) {
            Some(r) => r,
            None => return,
        };
        let txn = match self.journal.transactions.get(row.txn_idx) {
            Some(t) => t,
            None => return,
        };
        if txn.source_span.is_none() {
            return;
        }
        self.form = Some(FormState::from_transaction(txn));
        if let Some(f) = &mut self.form {
            f.update_completions(&self.acc.all);
        }
    }

    pub fn scroll_down(&mut self) {
        match self.view {
            View::Balances => {
                let n = self.bal.visible.len();
                if n == 0 {
                    return;
                }
                let i = self.bal.list_state.selected().map_or(0, |i| (i + 1).min(n - 1));
                self.bal.list_state.select(Some(i));
            }
            View::Register => {
                let n = self.reg.rows.len();
                if n == 0 {
                    return;
                }
                let i = self.reg.table_state.selected().map_or(0, |i| (i + 1).min(n - 1));
                self.reg.table_state.select(Some(i));
            }
            View::Accounts => {
                let n = self.acc.filtered.len();
                if n == 0 {
                    return;
                }
                let i = self.acc.list_state.selected().map_or(0, |i| (i + 1).min(n - 1));
                self.acc.list_state.select(Some(i));
            }
        }
    }

    pub fn scroll_up(&mut self) {
        match self.view {
            View::Balances => {
                let i = self.bal.list_state.selected().map_or(0, |i| i.saturating_sub(1));
                self.bal.list_state.select(Some(i));
            }
            View::Register => {
                let i = self.reg.table_state.selected().map_or(0, |i| i.saturating_sub(1));
                self.reg.table_state.select(Some(i));
            }
            View::Accounts => {
                let i = self.acc.list_state.selected().map_or(0, |i| i.saturating_sub(1));
                self.acc.list_state.select(Some(i));
            }
        }
    }

    pub fn goto_top(&mut self) {
        match self.view {
            View::Balances => self.bal.list_state.select(Some(0)),
            View::Register => self.reg.table_state.select(Some(0)),
            View::Accounts => self.acc.list_state.select(Some(0)),
        }
    }

    pub fn goto_bottom(&mut self) {
        match self.view {
            View::Balances => {
                let n = self.bal.visible.len();
                if n > 0 {
                    self.bal.list_state.select(Some(n - 1));
                }
            }
            View::Register => {
                let n = self.reg.rows.len();
                if n > 0 {
                    self.reg.table_state.select(Some(n - 1));
                }
            }
            View::Accounts => {
                let n = self.acc.filtered.len();
                if n > 0 {
                    self.acc.list_state.select(Some(n - 1));
                }
            }
        }
    }

    pub fn toggle_collapse(&mut self) {
        if self.view != View::Balances {
            return;
        }
        let vis_idx = match self.bal.list_state.selected() {
            Some(i) => i,
            None => return,
        };
        let row_idx = match self.bal.visible.get(vis_idx) {
            Some(&i) => i,
            None => return,
        };
        let acc_str = self.bal.report.rows[row_idx].account.as_str();
        let has_children = self.bal.report.rows.iter().any(|r| {
            let s = r.account.as_str();
            s != acc_str && s.starts_with(&format!("{acc_str}:"))
        });
        if !has_children {
            return;
        }
        if self.bal.collapsed.contains(&acc_str) {
            self.bal.collapsed.remove(&acc_str);
        } else {
            self.bal.collapsed.insert(acc_str);
        }
        let new_vis = compute_visible(&self.bal.report, &self.bal.collapsed);
        self.bal.visible = new_vis;
        let n = self.bal.visible.len();
        self.bal.list_state.select(Some(vis_idx.min(n.saturating_sub(1))));
    }

    pub fn drill_into_register(&mut self) {
        let acc = match self.view {
            View::Accounts => {
                let vi = match self.acc.list_state.selected() { Some(i) => i, None => return };
                let ai = match self.acc.filtered.get(vi) { Some(&i) => i, None => return };
                self.acc.all[ai].as_str()
            }
            View::Balances => {
                let vi = match self.bal.list_state.selected() { Some(i) => i, None => return };
                let ri = match self.bal.visible.get(vi) { Some(&i) => i, None => return };
                self.bal.report.rows[ri].account.as_str()
            }
            View::Register => return,
        };
        self.drill_stack.push((self.view, self.reg.account_filter.clone()));
        self.reg.account_filter = Some(acc);
        self.refresh_reg();
        self.view = View::Register;
        self.reg.table_state.select(Some(0));
    }

    pub fn go_back(&mut self) {
        if let Some((prev_view, prev_filter)) = self.drill_stack.pop() {
            self.view = prev_view;
            self.reg.account_filter = prev_filter;
            self.refresh_reg();
        } else if !self.filter.is_empty() {
            self.filter.clear();
            self.apply_filter();
        }
        self.filtering = false;
    }

    pub fn switch_view(&mut self, view: View) {
        if self.view == view {
            return;
        }
        self.drill_stack.clear();
        self.reg.account_filter = None;
        self.view = view;
        self.filtering = false;
        self.filter.clear();
        self.refresh_reg();
        self.rebuild_bal();
        self.rebuild_acc();
    }

    pub fn start_filter(&mut self) {
        self.filtering = true;
    }

    pub fn stop_filter(&mut self) {
        self.filtering = false;
    }

    pub fn cancel_filter(&mut self) {
        self.filtering = false;
        self.filter.clear();
        self.apply_filter();
    }

    pub fn push_char(&mut self, c: char) {
        self.filter.push(c);
        self.apply_filter();
    }

    pub fn pop_char(&mut self) {
        if self.filter.pop().is_some() {
            self.apply_filter();
        }
    }

    fn apply_filter(&mut self) {
        match self.view {
            View::Balances => self.rebuild_bal(),
            View::Register => {
                let fl = self.filter.to_lowercase();
                let q = Query {
                    account: self.reg.account_filter.clone(),
                    payee: if fl.is_empty() { None } else { Some(self.filter.clone()) },
                    ..Default::default()
                };
                let rep = report::register(&self.journal, &q);
                self.reg.rows = rep.rows;
                let n = self.reg.rows.len();
                self.reg.table_state.select(if n > 0 { Some(0) } else { None });
            }
            View::Accounts => self.rebuild_acc(),
        }
    }

    fn rebuild_bal(&mut self) {
        let fl = self.filter.to_lowercase();
        let q = Query {
            account: if fl.is_empty() { None } else { Some(self.filter.clone()) },
            ..Default::default()
        };
        let rep = report::balance(&self.journal, &q);
        let vis = compute_visible(&rep, &self.bal.collapsed);
        let n = vis.len();
        self.bal.report = rep;
        self.bal.visible = vis;
        self.bal.list_state.select(if n > 0 { Some(0) } else { None });
    }

    fn rebuild_acc(&mut self) {
        let fl = self.filter.to_lowercase();
        self.acc.filtered = self
            .acc
            .all
            .iter()
            .enumerate()
            .filter(|(_, a)| a.as_str().to_lowercase().contains(&fl))
            .map(|(i, _)| i)
            .collect();
        let n = self.acc.filtered.len();
        self.acc.list_state.select(if n > 0 { Some(0) } else { None });
    }

    fn refresh_reg(&mut self) {
        let q = Query {
            account: self.reg.account_filter.clone(),
            ..Default::default()
        };
        let rep = report::register(&self.journal, &q);
        self.reg.rows = rep.rows;
        let n = self.reg.rows.len();
        self.reg.table_state.select(if n > 0 { Some(0) } else { None });
    }
}

impl App {
    fn current_len(&self) -> usize {
        match self.view {
            View::Balances => self.bal.visible.len(),
            View::Register => self.reg.rows.len(),
            View::Accounts => self.acc.filtered.len(),
        }
    }

    fn selected_index(&self) -> Option<usize> {
        match self.view {
            View::Balances => self.bal.list_state.selected(),
            View::Register => self.reg.table_state.selected(),
            View::Accounts => self.acc.list_state.selected(),
        }
    }

    fn selected_offset(&self) -> usize {
        match self.view {
            View::Balances => self.bal.list_state.offset(),
            View::Register => self.reg.table_state.offset(),
            View::Accounts => self.acc.list_state.offset(),
        }
    }

    fn set_selected(&mut self, i: usize) {
        match self.view {
            View::Balances => self.bal.list_state.select(Some(i)),
            View::Register => self.reg.table_state.select(Some(i)),
            View::Accounts => self.acc.list_state.select(Some(i)),
        }
    }

    pub fn scroll_by(&mut self, delta: isize) {
        let len = self.current_len();
        if len == 0 {
            return;
        }
        let cur = self.selected_index().unwrap_or(0) as isize;
        let new = (cur + delta).clamp(0, len as isize - 1) as usize;
        self.set_selected(new);
    }

    pub fn page(&mut self, down: bool, half: bool) {
        let mut step = self.viewport_height.max(1) as isize;
        if half {
            step = (step / 2).max(1);
        }
        self.scroll_by(if down { step } else { -step });
    }

    pub fn click_select(&mut self, screen_row: u16) {
        if screen_row < self.list_top {
            return;
        }
        let rel = (screen_row - self.list_top) as usize;
        let idx = self.selected_offset() + rel;
        if idx < self.current_len() {
            self.set_selected(idx);
        }
    }

    pub fn collapse_all(&mut self) {
        if self.view != View::Balances {
            return;
        }
        let all: Vec<String> = self.bal.report.rows.iter().map(|r| r.account.as_str()).collect();
        self.bal.collapsed.clear();
        for s in &all {
            let has_children = all.iter().any(|o| o != s && o.starts_with(&format!("{s}:")));
            if has_children {
                self.bal.collapsed.insert(s.clone());
            }
        }
        self.bal.visible = compute_visible(&self.bal.report, &self.bal.collapsed);
        let n = self.bal.visible.len();
        self.bal.list_state.select(if n > 0 { Some(0) } else { None });
    }

    pub fn expand_all(&mut self) {
        if self.view != View::Balances {
            return;
        }
        self.bal.collapsed.clear();
        self.bal.visible = compute_visible(&self.bal.report, &self.bal.collapsed);
        let n = self.bal.visible.len();
        self.bal.list_state.select(if n > 0 { Some(0) } else { None });
    }

    pub fn result_count(&self) -> usize {
        self.current_len()
    }

    pub fn net_worth(&self) -> String {
        use tally_core::model::{Amount, Commodity};
        let mut totals: HashMap<String, (rust_decimal::Decimal, Commodity)> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        for txn in &self.journal.transactions {
            for p in &txn.postings {
                let top = p.account.top_level();
                if top == "Assets" || top == "Liabilities" {
                    if let Some(a) = &p.amount {
                        let key = a.commodity.symbol.clone();
                        if !totals.contains_key(&key) {
                            order.push(key.clone());
                        }
                        let e = totals
                            .entry(key)
                            .or_insert_with(|| (rust_decimal::Decimal::ZERO, a.commodity.clone()));
                        e.0 += a.quantity;
                    }
                }
            }
        }
        let parts: Vec<String> = order
            .iter()
            .filter_map(|k| totals.get(k))
            .filter(|(q, _)| *q != rust_decimal::Decimal::ZERO)
            .map(|(q, c)| tally_core::printer::format_amount(&Amount::new(*q, c.clone())))
            .collect();
        if parts.is_empty() {
            "0".to_string()
        } else {
            parts.join(", ")
        }
    }
}

pub fn run(journal: Journal, path: PathBuf) -> miette::Result<()> {
    install_panic_hook();

    enable_raw_mode().map_err(|e| miette!("terminal: {e}"))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| miette!("{e}"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| miette!("{e}"))?;

    let mut app = App::new(journal, path);
    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).ok();
    terminal.show_cursor().ok();

    result
}

fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> miette::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app)).map_err(|e| miette!("{e}"))?;

        if event::poll(Duration::from_millis(200)).map_err(|e| miette!("{e}"))? {
            match event::read().map_err(|e| miette!("{e}"))? {
                Event::Key(key) => {
                    if !handle_key(app, key) {
                        return Ok(());
                    }
                }
                Event::Mouse(me) => handle_mouse(app, me),
                _ => {}
            }
        }
    }
}

fn handle_mouse(app: &mut App, me: MouseEvent) {
    if app.form.is_some() || app.show_help {
        return;
    }
    match me.kind {
        MouseEventKind::ScrollDown => app.scroll_by(1),
        MouseEventKind::ScrollUp => app.scroll_by(-1),
        MouseEventKind::Down(MouseButton::Left) => app.click_select(me.row),
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }

    if app.form.is_some() {
        return handle_form_key(app, key);
    }

    if app.show_help {
        app.show_help = false;
        return true;
    }

    if app.filtering {
        match key.code {
            KeyCode::Esc => app.cancel_filter(),
            KeyCode::Enter => app.stop_filter(),
            KeyCode::Backspace => app.pop_char(),
            KeyCode::Char(c) => app.push_char(c),
            _ => {}
        }
        return true;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('d') => {
                app.page(true, true);
                return true;
            }
            KeyCode::Char('u') => {
                app.page(false, true);
                return true;
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return false,
        KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
        KeyCode::PageDown => app.page(true, false),
        KeyCode::PageUp => app.page(false, false),
        KeyCode::Char('[') => app.collapse_all(),
        KeyCode::Char(']') => app.expand_all(),
        KeyCode::Char('g') => app.goto_top(),
        KeyCode::Char('G') => app.goto_bottom(),
        KeyCode::Char('1') | KeyCode::Char('b') => app.switch_view(View::Balances),
        KeyCode::Char('2') | KeyCode::Char('r') => app.switch_view(View::Register),
        KeyCode::Char('3') | KeyCode::Char('a') => app.switch_view(View::Accounts),
        KeyCode::Tab => {
            let next = match app.view {
                View::Balances => View::Register,
                View::Register => View::Accounts,
                View::Accounts => View::Balances,
            };
            app.switch_view(next);
        }
        KeyCode::Char('/') => app.start_filter(),
        KeyCode::Char('?') | KeyCode::Char('h') => app.show_help = !app.show_help,
        KeyCode::Enter => app.drill_into_register(),
        KeyCode::Esc | KeyCode::Char('u') => app.go_back(),
        KeyCode::Char(' ') | KeyCode::Char('o') => app.toggle_collapse(),
        KeyCode::Char('n') => app.open_new_form(),
        KeyCode::Char('e') => app.open_edit_form(),
        _ => {}
    }
    true
}

fn handle_form_key(app: &mut App, key: KeyEvent) -> bool {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    if ctrl {
        match key.code {
            KeyCode::Char('s') => {
                try_save(app);
                return true;
            }
            KeyCode::Char('n') => {
                if let Some(f) = &mut app.form {
                    f.add_posting();
                    let accounts = app.acc.all.clone();
                    if let Some(f) = &mut app.form {
                        f.update_completions(&accounts);
                    }
                }
                return true;
            }
            KeyCode::Char('d') => {
                if let Some(f) = &mut app.form {
                    f.delete_posting();
                    f.update_balance();
                }
                return true;
            }
            _ => {}
        }
    }

    let Some(form) = &mut app.form else { return true };

    match key.code {
        KeyCode::Esc => {
            if form.completion_open {
                form.completion_open = false;
            } else {
                app.form = None;
            }
            return true;
        }
        KeyCode::Tab if shift => {
            form.tab_prev();
            let accounts = app.acc.all.clone();
            if let Some(f) = &mut app.form {
                f.update_completions(&accounts);
                if matches!(f.focus, Focus::PostingAccount(_)) {
                    f.completion_open = !f.completions.is_empty();
                }
            }
            return true;
        }
        KeyCode::Tab => {
            form.tab_next();
            let accounts = app.acc.all.clone();
            if let Some(f) = &mut app.form {
                f.update_completions(&accounts);
                if matches!(f.focus, Focus::PostingAccount(_)) {
                    f.completion_open = !f.completions.is_empty();
                }
            }
            return true;
        }
        _ => {}
    }

    let Some(form) = &mut app.form else { return true };

    if form.completion_open {
        match key.code {
            KeyCode::Up => {
                if form.completion_sel > 0 {
                    form.completion_sel -= 1;
                }
                return true;
            }
            KeyCode::Down => {
                if form.completion_sel + 1 < form.completions.len() {
                    form.completion_sel += 1;
                }
                return true;
            }
            KeyCode::Enter => {
                form.select_completion();
                form.update_balance();
                return true;
            }
            _ => {
                form.completion_open = false;
            }
        }
    }

    match &form.focus.clone() {
        Focus::Date => {
            if key.code == KeyCode::Enter {
                form.tab_next();
            } else {
                form.date.handle(key.code);
            }
        }
        Focus::Status => {
            if key.code == KeyCode::Char(' ') || key.code == KeyCode::Enter {
                form.cycle_status();
            }
        }
        Focus::Payee => {
            if key.code == KeyCode::Enter {
                form.tab_next();
            } else {
                form.payee.handle(key.code);
            }
        }
        Focus::PostingAccount(i) => {
            let i = *i;
            form.postings[i].account.handle(key.code);
            let accounts = app.acc.all.clone();
            if let Some(f) = &mut app.form {
                f.update_completions(&accounts);
                f.completion_open = !f.completions.is_empty()
                    && !f.postings[i].account.text.is_empty();
                f.update_balance();
            }
            return true;
        }
        Focus::PostingAmount(i) => {
            let i = *i;
            if key.code == KeyCode::Enter {
                form.tab_next();
            } else {
                form.postings[i].amount.handle(key.code);
                form.update_balance();
            }
        }
    }

    true
}

fn try_save(app: &mut App) {
    let form = match &app.form {
        Some(f) => f,
        None => return,
    };

    let txn = match form.try_build() {
        Ok(t) => t,
        Err(e) => {
            if let Some(f) = &mut app.form {
                f.error = Some(e);
            }
            return;
        }
    };

    let path = app.journal_path.clone();
    let form = app.form.as_ref().unwrap();

    match form::save_and_reload(form, &txn, &path) {
        Ok(new_journal) => {
            app.form = None;
            app.reload(new_journal);
            app.view = View::Register;
        }
        Err(e) => {
            if let Some(f) = &mut app.form {
                f.error = Some(e);
            }
        }
    }
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        disable_raw_mode().ok();
        execute!(io::stdout(), LeaveAlternateScreen).ok();
        original(info);
    }));
}
