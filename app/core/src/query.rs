use jiff::civil::Date;

use crate::model::{Posting, Transaction};

#[derive(Debug, Default, Clone)]
pub struct Query {
    pub account: Option<String>,
    pub from: Option<Date>,
    pub to: Option<Date>,
    pub payee: Option<String>,
    pub tag: Option<String>,
}

impl Query {
    pub fn matches_txn(&self, txn: &Transaction) -> bool {
        if let Some(from) = self.from {
            if txn.date < from {
                return false;
            }
        }
        if let Some(to) = self.to {
            if txn.date > to {
                return false;
            }
        }
        if let Some(ref p) = self.payee {
            if !txn.payee.to_lowercase().contains(p.to_lowercase().as_str()) {
                return false;
            }
        }
        if let Some(ref tag) = self.tag {
            if !txn.tags.contains_key(tag.as_str()) {
                return false;
            }
        }
        true
    }

    pub fn matches_posting(&self, posting: &Posting) -> bool {
        if let Some(ref filter) = self.account {
            let acc = posting.account.as_str();
            if acc != *filter && !acc.starts_with(&format!("{filter}:")) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::Date;

    use super::*;
    use crate::model::{Account, Posting, Transaction};

    fn txn(date: Date, payee: &str) -> Transaction {
        Transaction::new(date, payee)
    }

    fn posting(account: &str) -> Posting {
        Posting::new(Account::parse(account), None)
    }

    #[test]
    fn date_range_filter() {
        let t = txn(Date::new(2026, 1, 15).unwrap(), "X");
        let q_before = Query {
            to: Some(Date::new(2026, 1, 14).unwrap()),
            ..Default::default()
        };
        let q_after = Query {
            from: Some(Date::new(2026, 1, 16).unwrap()),
            ..Default::default()
        };
        let q_match = Query {
            from: Some(Date::new(2026, 1, 1).unwrap()),
            to: Some(Date::new(2026, 1, 31).unwrap()),
            ..Default::default()
        };
        assert!(!q_before.matches_txn(&t));
        assert!(!q_after.matches_txn(&t));
        assert!(q_match.matches_txn(&t));
    }

    #[test]
    fn account_prefix_filter() {
        let p_assets = posting("Assets:Checking");
        let p_exp = posting("Expenses:Food");
        let q = Query {
            account: Some("Assets".to_string()),
            ..Default::default()
        };
        assert!(q.matches_posting(&p_assets));
        assert!(!q.matches_posting(&p_exp));
    }

    #[test]
    fn exact_account_match() {
        let p = posting("Assets:Checking");
        let q = Query {
            account: Some("Assets:Checking".to_string()),
            ..Default::default()
        };
        assert!(q.matches_posting(&p));
    }
}
