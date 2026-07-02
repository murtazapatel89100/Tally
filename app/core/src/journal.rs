use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use miette::Diagnostic;
use thiserror::Error;

use crate::error::ParseError;
use crate::model::{Account, Transaction};
use crate::parser::{self, Directive, Entry};

#[derive(Debug, Error, Diagnostic)]
pub enum JournalError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(#[from] ParseError),
    #[error("failed to read journal file `{path}`")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Default, Clone)]
pub struct Journal {
    pub transactions: Vec<Transaction>,
    pub accounts: IndexSet<Account>,
    pub commodities: IndexSet<String>,
    pub aliases: IndexMap<String, String>,
    pub warnings: Vec<String>,
}

impl Journal {
    pub fn parse_str(input: &str) -> Result<Journal, ParseError> {
        let entries = parser::parse(input, "<journal>")?;
        let mut journal = Journal::default();
        journal.build(entries);
        Ok(journal)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Journal, JournalError> {
        let entries = load_entries(path.as_ref())?;
        let mut journal = Journal::default();
        journal.build(entries);
        Ok(journal)
    }

    fn build(&mut self, entries: Vec<Entry>) {
        for entry in entries {
            match entry {
                Entry::Directive(directive) => self.apply_directive(directive),
                Entry::Transaction(mut txn) => {
                    for posting in &mut txn.postings {
                        posting.account = self.resolve_alias(&posting.account);
                        self.accounts.insert(posting.account.clone());
                        if let Some(amount) = &posting.amount
                            && !amount.commodity.symbol.is_empty()
                        {
                            self.commodities.insert(amount.commodity.symbol.clone());
                        }
                    }
                    self.transactions.push(txn);
                }
            }
        }
    }

    fn apply_directive(&mut self, directive: Directive) {
        match directive {
            Directive::Account(name) => {
                let account = self.resolve_alias(&Account::parse(&name));
                self.accounts.insert(account);
            }
            Directive::Commodity(symbol) => {
                self.commodities.insert(symbol);
            }
            Directive::Alias { old, new } => {
                self.aliases.insert(old, new);
            }
            Directive::Include(path) => {
                self.warnings.push(format!(
                    "`include {path}` ignored — load with `Journal::from_path` to resolve includes"
                ));
            }
            Directive::Other(name) => {
                self.warnings
                    .push(format!("unsupported directive `{name}` skipped"));
            }
        }
    }

    fn resolve_alias(&self, account: &Account) -> Account {
        let full = account.as_str();
        if let Some(new) = self.aliases.get(&full) {
            return Account::parse(new);
        }
        for (old, new) in &self.aliases {
            if let Some(rest) = full.strip_prefix(old.as_str())
                && rest.starts_with(':')
            {
                return Account::parse(&format!("{new}{rest}"));
            }
        }
        account.clone()
    }
}

fn load_entries(path: &Path) -> Result<Vec<Entry>, JournalError> {
    let text = std::fs::read_to_string(path).map_err(|source| JournalError::Io {
        path: path.display().to_string(),
        source,
    })?;
    let name = path.display().to_string();
    let entries = parser::parse(&text, &name)?;
    let base = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut out = Vec::new();
    for entry in entries {
        match entry {
            Entry::Directive(Directive::Include(rel)) => {
                out.extend(load_entries(&base.join(&rel))?);
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn indexes_accounts_and_commodities_in_order() {
        let journal = Journal::parse_str(
            "2026-01-01 * Opening\n    Assets:Checking    $5\n    Equity:Opening\n\n2026-01-02 * Buy AAPL\n    Assets:Broker    10 AAPL\n    Assets:Checking    -$5\n",
        )
        .unwrap();

        assert_eq!(journal.transactions.len(), 2);
        let accounts: Vec<String> = journal.accounts.iter().map(|a| a.as_str()).collect();
        assert_eq!(
            accounts,
            vec!["Assets:Checking", "Equity:Opening", "Assets:Broker"]
        );
        let commodities: Vec<&str> = journal.commodities.iter().map(String::as_str).collect();
        assert_eq!(commodities, vec!["$", "AAPL"]);
    }

    #[test]
    fn resolves_aliases_on_postings() {
        let journal = Journal::parse_str(
            "alias Cash = Assets:Checking\n2026-01-01 * Coffee\n    Expenses:Coffee    $4\n    Cash\n",
        )
        .unwrap();

        let last = &journal.transactions[0].postings[1];
        assert_eq!(last.account.as_str(), "Assets:Checking");
        assert_eq!(last.amount.as_ref().unwrap().quantity, dec!(-4));
        assert!(
            journal
                .accounts
                .contains(&Account::parse("Assets:Checking"))
        );
    }

    #[test]
    fn resolves_alias_prefix_segments() {
        let journal = Journal::parse_str(
            "alias A = Assets\n2026-01-01 * Move\n    A:Checking    $1\n    A:Savings    -$1\n",
        )
        .unwrap();
        assert_eq!(
            journal.transactions[0].postings[0].account.as_str(),
            "Assets:Checking"
        );
        assert_eq!(
            journal.transactions[0].postings[1].account.as_str(),
            "Assets:Savings"
        );
    }

    #[test]
    fn unresolved_include_becomes_a_warning() {
        let journal = Journal::parse_str("include more.journal\n").unwrap();
        assert_eq!(journal.transactions.len(), 0);
        assert_eq!(journal.warnings.len(), 1);
        assert!(journal.warnings[0].contains("include more.journal"));
    }

    #[test]
    fn unsupported_directive_warns_but_does_not_fail() {
        let journal = Journal::parse_str("year 2026\n2026-01-01 * X\n    A  $1\n    B\n").unwrap();
        assert_eq!(journal.transactions.len(), 1);
        assert!(journal.warnings.iter().any(|w| w.contains("year")));
    }
}
