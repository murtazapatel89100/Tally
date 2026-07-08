use std::collections::BTreeMap;

use indexmap::IndexMap;
use jiff::civil::Date;
use rust_decimal::Decimal;

use crate::{
    journal::Journal,
    model::{Account, Amount, Commodity},
    printer::format_amount,
    query::Query,
};

pub struct BalRow {
    pub account: Account,
    pub amounts: Vec<Amount>,
    pub depth: usize,
}

pub struct BalReport {
    pub rows: Vec<BalRow>,
    pub totals: Vec<Amount>,
}

pub struct RegRow {
    pub date: Date,
    pub payee: String,
    pub account: Account,
    pub amount: Option<Amount>,
    pub running: Vec<Amount>,
    pub txn_idx: usize,
}

pub struct RegReport {
    pub rows: Vec<RegRow>,
}

type AmountMap = IndexMap<String, (Decimal, Commodity)>;

fn amtmap_add(map: &mut AmountMap, amount: &Amount) {
    let e = map
        .entry(amount.commodity.symbol.clone())
        .or_insert_with(|| (Decimal::ZERO, amount.commodity.clone()));
    e.0 += amount.quantity;
}

fn amtmap_to_vec(map: &AmountMap) -> Vec<Amount> {
    map.values()
        .filter(|(q, _)| *q != Decimal::ZERO)
        .map(|(q, c)| Amount::new(*q, c.clone()))
        .collect()
}

fn amtmap_to_vec_all(map: &AmountMap) -> Vec<Amount> {
    map.values()
        .map(|(q, c)| Amount::new(*q, c.clone()))
        .collect()
}

pub fn balance(journal: &Journal, query: &Query) -> BalReport {
    let mut own: BTreeMap<String, AmountMap> = BTreeMap::new();

    for txn in &journal.transactions {
        if !query.matches_txn(txn) {
            continue;
        }
        for posting in &txn.postings {
            if !query.matches_posting(posting) {
                continue;
            }
            if let Some(amount) = &posting.amount {
                amtmap_add(own.entry(posting.account.as_str()).or_default(), amount);
            }
        }
    }

    let mut all: BTreeMap<String, ()> = BTreeMap::new();
    for acc_str in own.keys() {
        let parts: Vec<&str> = acc_str.split(':').collect();
        for d in 1..=parts.len() {
            all.insert(parts[..d].join(":"), ());
        }
    }

    let all_keys: Vec<String> = all.into_keys().collect();
    let mut rows = Vec::new();

    for acc_str in &all_keys {
        let mut total: AmountMap = IndexMap::new();
        for (other, amounts) in &own {
            if other == acc_str || other.starts_with(&format!("{acc_str}:")) {
                for (_, amount) in amounts {
                    amtmap_add(&mut total, &Amount::new(amount.0, amount.1.clone()));
                }
            }
        }
        let amounts = amtmap_to_vec(&total);
        if amounts.is_empty() {
            continue;
        }
        let acc = Account::parse(acc_str);
        let depth = acc.depth() - 1;
        rows.push(BalRow {
            account: acc,
            amounts,
            depth,
        });
    }

    let mut grand: AmountMap = IndexMap::new();
    for amounts in own.values() {
        for (_, amount) in amounts {
            amtmap_add(&mut grand, &Amount::new(amount.0, amount.1.clone()));
        }
    }

    BalReport {
        rows,
        totals: amtmap_to_vec(&grand),
    }
}

pub fn register(journal: &Journal, query: &Query) -> RegReport {
    let mut running: AmountMap = IndexMap::new();
    let mut rows = Vec::new();

    for (txn_idx, txn) in journal.transactions.iter().enumerate() {
        if !query.matches_txn(txn) {
            continue;
        }
        for posting in &txn.postings {
            if !query.matches_posting(posting) {
                continue;
            }
            if let Some(amount) = &posting.amount {
                amtmap_add(&mut running, amount);
            }
            rows.push(RegRow {
                date: txn.date,
                payee: txn.payee.clone(),
                account: posting.account.clone(),
                amount: posting.amount.clone(),
                running: amtmap_to_vec_all(&running),
                txn_idx,
            });
        }
    }

    RegReport { rows }
}

impl BalReport {
    pub fn render(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }

        let amt_width = self
            .rows
            .iter()
            .flat_map(|r| r.amounts.iter())
            .map(|a| format_amount(a).len())
            .chain(self.totals.iter().map(|a| format_amount(a).len()))
            .max()
            .unwrap_or(10)
            .max(10);

        let mut out = String::new();

        for row in &self.rows {
            let indent = "  ".repeat(row.depth);
            let display = row.account.0.last().map(String::as_str).unwrap_or("");
            for (i, amount) in row.amounts.iter().enumerate() {
                let amt_str = format_amount(amount);
                if i == 0 {
                    out.push_str(&format!(
                        "{:>width$}  {indent}{display}\n",
                        amt_str,
                        width = amt_width
                    ));
                } else {
                    let pad = " ".repeat(indent.len() + display.len());
                    out.push_str(&format!("{:>width$}  {pad}\n", amt_str, width = amt_width));
                }
            }
        }

        out.push_str(&format!("{}\n", "-".repeat(amt_width + 2)));

        if self.totals.is_empty() {
            out.push_str(&format!("{:>width$}\n", "0", width = amt_width));
        } else {
            for amount in &self.totals {
                out.push_str(&format!(
                    "{:>width$}\n",
                    format_amount(amount),
                    width = amt_width
                ));
            }
        }

        out
    }
}

impl RegReport {
    pub fn render(&self) -> String {
        const DATE_W: usize = 10;
        const PAYEE_W: usize = 24;
        const ACCT_W: usize = 26;
        const AMT_W: usize = 13;

        let mut out = String::new();
        for row in &self.rows {
            let date = row.date.to_string();
            let payee = trunc(&row.payee, PAYEE_W);
            let account = trunc(&row.account.as_str(), ACCT_W);
            let amt = row.amount.as_ref().map(format_amount).unwrap_or_default();
            let run = if row.running.is_empty() {
                "0".to_string()
            } else {
                row.running
                    .iter()
                    .map(format_amount)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            out.push_str(&format!(
                "{:<DATE_W$} {:<PAYEE_W$} {:<ACCT_W$} {:>AMT_W$} {:>AMT_W$}\n",
                date, payee, account, amt, run,
            ));
        }
        out
    }
}

fn trunc(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::Journal;

    fn sample() -> Journal {
        Journal::parse_str(
            "2026-01-01 * Opening\n    Assets:Checking    $5,000.00\n    Equity:Opening\n\
             2026-01-05 * Groceries\n    Expenses:Food    $100.00\n    Assets:Checking\n",
        )
        .unwrap()
    }

    #[test]
    fn balance_totals_by_account() {
        let j = sample();
        let rep = balance(&j, &Query::default());
        let checking = rep
            .rows
            .iter()
            .find(|r| r.account.as_str() == "Assets:Checking")
            .unwrap();
        assert_eq!(
            checking.amounts[0].quantity,
            rust_decimal_macros::dec!(4900.00)
        );
    }

    #[test]
    fn register_running_total() {
        let j = sample();
        let rep = register(&j, &Query::default());
        assert!(!rep.rows.is_empty());
        let last = rep.rows.last().unwrap();
        assert!(!last.running.is_empty());
        assert_eq!(last.running[0].quantity, rust_decimal_macros::dec!(0));
    }

    #[test]
    fn balance_account_filter() {
        let j = sample();
        let q = Query {
            account: Some("Assets".to_string()),
            ..Default::default()
        };
        let rep = balance(&j, &q);
        assert!(
            rep.rows
                .iter()
                .all(|r| r.account.as_str().starts_with("Assets"))
        );
    }
}
