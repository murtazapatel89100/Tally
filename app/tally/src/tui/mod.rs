use std::collections::HashSet;
use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
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

pub mod ui;

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
    pub view: View,
    pub filter: String,
    pub filtering: bool,
    pub show_help: bool,
    pub bal: BalState,
    pub reg: RegState,
    pub acc: AccState,
    pub drill_stack: Vec<(View, Option<String>)>,
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
    pub fn new(journal: Journal) -> Self {
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
                let vi = match self.acc.list_state.selected() {
                    Some(i) => i,
                    None => return,
                };
                let ai = match self.acc.filtered.get(vi) {
                    Some(&i) => i,
                    None => return,
                };
                self.acc.all[ai].as_str()
            }
            View::Balances => {
                let vi = match self.bal.list_state.selected() {
                    Some(i) => i,
                    None => return,
                };
                let ri = match self.bal.visible.get(vi) {
                    Some(&i) => i,
                    None => return,
                };
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
        let fl = self.filter.to_lowercase();
        match self.view {
            View::Balances => self.rebuild_bal(),
            View::Register => {
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
        self.acc.filtered = self.acc
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

pub fn run(journal: Journal) -> miette::Result<()> {
    install_panic_hook();

    enable_raw_mode().map_err(|e| miette!("terminal: {e}"))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| miette!("{e}"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| miette!("{e}"))?;

    let mut app = App::new(journal);
    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
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
            if let Event::Key(key) = event::read().map_err(|e| miette!("{e}"))? {
                if !handle_key(app, key) {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
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

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return false,
        KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
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
        _ => {}
    }
    true
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        disable_raw_mode().ok();
        execute!(io::stdout(), LeaveAlternateScreen).ok();
        original(info);
    }));
}
