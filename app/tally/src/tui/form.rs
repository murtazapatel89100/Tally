use std::{collections::HashMap, fs, io::Write as _, path::Path};

use crossterm::event::KeyCode;
use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};
use rust_decimal::Decimal;
use tally_core::{
    journal::Journal,
    model::{Account, Amount, Commodity, Posting, SourceSpan, Status, Transaction},
    parser::{parse_amount, parse_date},
    printer::{format_amount, print_transaction},
};

#[derive(Debug, Clone)]
pub enum FormMode {
    New,
    Edit { span: SourceSpan },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Date,
    Status,
    Payee,
    PostingAccount(usize),
    PostingAmount(usize),
}

#[derive(Debug, Default, Clone)]
pub struct TextInput {
    pub text: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn with_text(s: impl Into<String>) -> Self {
        let text = s.into();
        let cursor = text.len();
        Self { text, cursor }
    }

    pub fn handle(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => {
                self.text.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let p = prev_boundary(&self.text, self.cursor);
                    self.text.drain(p..self.cursor);
                    self.cursor = p;
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.text.len() {
                    let n = next_boundary(&self.text, self.cursor);
                    self.text.drain(self.cursor..n);
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor = prev_boundary(&self.text, self.cursor);
                }
            }
            KeyCode::Right => {
                if self.cursor < self.text.len() {
                    self.cursor = next_boundary(&self.text, self.cursor);
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.text.len(),
            _ => {}
        }
    }
}

fn prev_boundary(s: &str, pos: usize) -> usize {
    (0..pos).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0)
}

fn next_boundary(s: &str, pos: usize) -> usize {
    (pos + 1..=s.len())
        .find(|&i| s.is_char_boundary(i))
        .unwrap_or(s.len())
}

#[derive(Debug, Default, Clone)]
pub struct PostingRow {
    pub account: TextInput,
    pub amount: TextInput,
}

pub struct FormState {
    pub mode: FormMode,
    pub date: TextInput,
    pub status: Status,
    pub payee: TextInput,
    pub postings: Vec<PostingRow>,
    pub focus: Focus,
    pub completions: Vec<String>,
    pub completion_sel: usize,
    pub completion_open: bool,
    pub balance_ok: bool,
    pub balance_note: String,
    pub error: Option<String>,
}

impl FormState {
    pub fn new_transaction() -> Self {
        let today = jiff::Zoned::now().date().to_string();
        let mut s = Self {
            mode: FormMode::New,
            date: TextInput::with_text(today),
            status: Status::Uncleared,
            payee: TextInput::default(),
            postings: vec![PostingRow::default(), PostingRow::default()],
            focus: Focus::Payee,
            completions: Vec::new(),
            completion_sel: 0,
            completion_open: false,
            balance_ok: false,
            balance_note: "no postings".into(),
            error: None,
        };
        s.update_balance();
        s
    }

    pub fn from_transaction(txn: &Transaction) -> Self {
        let span = txn.source_span.clone().unwrap_or_default();
        let postings = txn
            .postings
            .iter()
            .map(|p| PostingRow {
                account: TextInput::with_text(p.account.as_str()),
                amount: TextInput::with_text(
                    p.amount.as_ref().map(format_amount).unwrap_or_default(),
                ),
            })
            .collect();

        let mut s = Self {
            mode: FormMode::Edit { span },
            date: TextInput::with_text(txn.date.to_string()),
            status: txn.status,
            payee: TextInput::with_text(txn.payee.clone()),
            postings,
            focus: Focus::Payee,
            completions: Vec::new(),
            completion_sel: 0,
            completion_open: false,
            balance_ok: false,
            balance_note: String::new(),
            error: None,
        };
        s.update_balance();
        s
    }

    pub fn update_balance(&mut self) {
        let mut blanks = 0usize;
        let mut totals: HashMap<String, (Decimal, Commodity)> = HashMap::new();

        for p in &self.postings {
            if p.account.text.trim().is_empty() {
                continue;
            }
            if p.amount.text.trim().is_empty() {
                blanks += 1;
            } else if let Some(a) = parse_amount(p.amount.text.trim()) {
                let entry = totals
                    .entry(a.commodity.symbol.clone())
                    .or_insert_with(|| (Decimal::ZERO, a.commodity.clone()));
                entry.0 += a.quantity;
            }
        }

        if blanks > 1 {
            self.balance_ok = false;
            self.balance_note = format!("{blanks} blank postings (max 1)");
        } else if blanks == 1 {
            self.balance_ok = true;
            self.balance_note = "✓ one blank — will be inferred".into();
        } else if totals.is_empty() {
            self.balance_ok = false;
            self.balance_note = "no postings".into();
        } else {
            let off: Vec<String> = totals
                .values()
                .filter(|(q, _)| *q != Decimal::ZERO)
                .map(|(q, c)| format_amount(&Amount::new(*q, c.clone())))
                .collect();
            if off.is_empty() {
                self.balance_ok = true;
                self.balance_note = "✓ balanced".into();
            } else {
                self.balance_ok = false;
                self.balance_note = format!("⚠ off by {}", off.join(", "));
            }
        }
    }

    pub fn update_completions(&mut self, accounts: &[Account]) {
        let q = match &self.focus {
            Focus::PostingAccount(i) => self.postings[*i].account.text.clone(),
            _ => return,
        };
        self.completions = fuzzy_accounts(&q, accounts);
        self.completion_sel = 0;
    }

    pub fn select_completion(&mut self) {
        if !self.completion_open {
            return;
        }
        let Some(acc) = self.completions.get(self.completion_sel).cloned() else {
            return;
        };
        if let Focus::PostingAccount(i) = self.focus {
            let len = acc.len();
            self.postings[i].account.text = acc;
            self.postings[i].account.cursor = len;
        }
        self.completion_open = false;
    }

    pub fn cycle_status(&mut self) {
        self.status = match self.status {
            Status::Uncleared => Status::Cleared,
            Status::Cleared => Status::Pending,
            Status::Pending => Status::Uncleared,
        };
    }

    pub fn tab_next(&mut self) {
        self.completion_open = false;
        self.focus = match &self.focus {
            Focus::Date => Focus::Status,
            Focus::Status => Focus::Payee,
            Focus::Payee => Focus::PostingAccount(0),
            Focus::PostingAccount(i) => Focus::PostingAmount(*i),
            Focus::PostingAmount(i) => {
                let next = i + 1;
                if next >= self.postings.len() {
                    if self.postings.len() < 8 {
                        self.postings.push(PostingRow::default());
                    }
                    Focus::PostingAccount(self.postings.len() - 1)
                } else {
                    Focus::PostingAccount(next)
                }
            }
        };
    }

    pub fn tab_prev(&mut self) {
        self.completion_open = false;
        self.focus = match &self.focus {
            Focus::Date => Focus::Date,
            Focus::Status => Focus::Date,
            Focus::Payee => Focus::Status,
            Focus::PostingAccount(0) => Focus::Payee,
            Focus::PostingAccount(i) => Focus::PostingAmount(i - 1),
            Focus::PostingAmount(i) => Focus::PostingAccount(*i),
        };
    }

    pub fn add_posting(&mut self) {
        if self.postings.len() < 8 {
            self.postings.push(PostingRow::default());
            let i = self.postings.len() - 1;
            self.focus = Focus::PostingAccount(i);
        }
    }

    pub fn delete_posting(&mut self) {
        if self.postings.len() <= 2 {
            return;
        }
        let i = match &self.focus {
            Focus::PostingAccount(i) | Focus::PostingAmount(i) => *i,
            _ => return,
        };
        self.postings.remove(i);
        let new_i = i.min(self.postings.len() - 1);
        self.focus = Focus::PostingAccount(new_i);
    }

    pub fn try_build(&self) -> Result<Transaction, String> {
        let date = parse_date(&self.date.text)
            .ok_or_else(|| format!("invalid date '{}'", self.date.text))?;
        let payee = self.payee.text.trim();
        if payee.is_empty() {
            return Err("payee is required".into());
        }

        let mut txn = Transaction::new(date, payee);
        txn.status = self.status;

        let mut blanks = 0usize;
        for p in &self.postings {
            let acc = p.account.text.trim();
            if acc.is_empty() {
                continue;
            }
            let amount = if p.amount.text.trim().is_empty() {
                blanks += 1;
                if blanks > 1 {
                    return Err("only one blank posting allowed".into());
                }
                None
            } else {
                Some(
                    parse_amount(p.amount.text.trim())
                        .ok_or_else(|| format!("invalid amount '{}'", p.amount.text.trim()))?,
                )
            };
            txn.postings.push(Posting::new(Account::parse(acc), amount));
        }

        if txn.postings.is_empty() {
            return Err("at least one posting is required".into());
        }
        if !txn.is_balanced() {
            return Err("transaction does not balance".into());
        }
        Ok(txn)
    }
}

pub fn save_and_reload(
    form: &FormState,
    txn: &Transaction,
    path: &Path,
) -> Result<Journal, String> {
    let serialized = print_transaction(txn);

    match &form.mode {
        FormMode::New => {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(|e| e.to_string())?;
            file.write_all(b"\n").map_err(|e| e.to_string())?;
            file.write_all(serialized.as_bytes())
                .map_err(|e| e.to_string())?;
        }
        FormMode::Edit { span } => {
            let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let before = &content[..span.start];
            let after = &content[span.end..];
            let new_content = format!("{}{}{}", before, serialized.trim_end(), after);
            fs::write(path, new_content).map_err(|e| e.to_string())?;
        }
    }

    Journal::from_path(path).map_err(|e| e.to_string())
}

fn fuzzy_accounts(query: &str, accounts: &[Account]) -> Vec<String> {
    if accounts.is_empty() {
        return Vec::new();
    }
    if query.is_empty() {
        return accounts.iter().map(|a| a.as_str()).take(10).collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let atom = Atom::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );

    let mut scored: Vec<(u16, String)> = accounts
        .iter()
        .map(|a| a.as_str())
        .filter_map(|s| {
            let score = if s.is_ascii() {
                atom.score(Utf32Str::Ascii(s.as_bytes()), &mut matcher)
            } else {
                let chars: Vec<char> = s.chars().collect();
                atom.score(Utf32Str::Unicode(&chars), &mut matcher)
            };
            score.map(|sc| (sc, s))
        })
        .collect();

    scored.sort_by_key(|&(sc, _)| std::cmp::Reverse(sc));
    scored.into_iter().take(8).map(|(_, s)| s).collect()
}
