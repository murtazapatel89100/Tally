use std::fmt;

use indexmap::IndexMap;
use jiff::civil::Date;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommodityPosition {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commodity {
    pub symbol: String,
    pub position: CommodityPosition,
}

impl Commodity {
    pub fn prefixed(symbol: impl Into<String>) -> Self {
        Self { symbol: symbol.into(), position: CommodityPosition::Prefix }
    }

    pub fn suffixed(symbol: impl Into<String>) -> Self {
        Self { symbol: symbol.into(), position: CommodityPosition::Suffix }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Amount {
    pub quantity: Decimal,
    pub commodity: Commodity,
}

impl Amount {
    pub fn new(quantity: Decimal, commodity: Commodity) -> Self {
        Self { quantity, commodity }
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        if self.commodity == other.commodity {
            Some(Self { quantity: self.quantity + other.quantity, commodity: self.commodity })
        } else {
            None
        }
    }
}

impl std::ops::Add for Amount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.checked_add(other).expect("cannot add amounts with different commodities")
    }
}

impl std::ops::Neg for Amount {
    type Output = Self;

    fn neg(self) -> Self {
        Self { quantity: -self.quantity, commodity: self.commodity }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Account(pub Vec<String>);

impl Account {
    pub fn parse(s: &str) -> Self {
        Self(s.split(':').map(str::to_owned).collect())
    }

    pub fn as_str(&self) -> String {
        self.0.join(":")
    }

    pub fn depth(&self) -> usize {
        self.0.len()
    }

    pub fn parent(&self) -> Option<Account> {
        if self.0.len() > 1 { Some(Account(self.0[..self.0.len() - 1].to_vec())) } else { None }
    }

    pub fn top_level(&self) -> &str {
        &self.0[0]
    }

    pub fn is_descendant_of(&self, other: &Account) -> bool {
        self.0.len() > other.0.len() && self.0.starts_with(&other.0)
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(":"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Uncleared,
    Pending,
    Cleared,
}

impl Status {
    pub fn marker(self) -> Option<char> {
        match self {
            Self::Uncleared => None,
            Self::Pending => Some('!'),
            Self::Cleared => Some('*'),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    pub account: Account,
    pub amount: Option<Amount>,
    pub status: Option<Status>,
    pub comment: Option<String>,
    pub tags: IndexMap<String, Option<String>>,
}

impl Posting {
    pub fn new(account: Account, amount: Option<Amount>) -> Self {
        Self { account, amount, status: None, comment: None, tags: IndexMap::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub date: Date,
    pub status: Status,
    pub code: Option<String>,
    pub payee: String,
    pub postings: Vec<Posting>,
    pub comment: Option<String>,
    pub tags: IndexMap<String, Option<String>>,
    pub source_span: Option<SourceSpan>,
}

impl Transaction {
    pub fn new(date: Date, payee: impl Into<String>) -> Self {
        Self {
            date,
            status: Status::default(),
            code: None,
            payee: payee.into(),
            postings: Vec::new(),
            comment: None,
            tags: IndexMap::new(),
            source_span: None,
        }
    }

    pub fn is_balanced(&self) -> bool {
        let inferred = self.postings.iter().filter(|p| p.amount.is_none()).count();
        if inferred > 1 {
            return false;
        }
        if inferred == 1 {
            return true;
        }
        let mut totals: IndexMap<String, Decimal> = IndexMap::new();
        for posting in &self.postings {
            if let Some(ref amount) = posting.amount {
                *totals.entry(amount.commodity.symbol.clone()).or_insert(Decimal::ZERO) +=
                    amount.quantity;
            }
        }
        totals.values().all(|&total| total == Decimal::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    fn usd(quantity: Decimal) -> Amount {
        Amount::new(quantity, Commodity::prefixed("$"))
    }

    #[test]
    fn amount_add_same_commodity() {
        let result = usd(dec!(10.00)) + usd(dec!(5.50));
        assert_eq!(result.quantity, dec!(15.50));
        assert_eq!(result.commodity, Commodity::prefixed("$"));
    }

    #[test]
    fn amount_checked_add_different_commodities_returns_none() {
        let a = Amount::new(dec!(10), Commodity::prefixed("$"));
        let b = Amount::new(dec!(10), Commodity::suffixed("EUR"));
        assert!(a.checked_add(b).is_none());
    }

    #[test]
    fn amount_neg() {
        let a = usd(dec!(42.00));
        assert_eq!((-a).quantity, dec!(-42.00));
    }

    #[test]
    fn account_parse_and_display() {
        let acc = Account::parse("Assets:Checking:Primary");
        assert_eq!(acc.depth(), 3);
        assert_eq!(acc.top_level(), "Assets");
        assert_eq!(acc.to_string(), "Assets:Checking:Primary");
    }

    #[test]
    fn account_is_descendant_of() {
        let parent = Account::parse("Expenses:Food");
        let child = Account::parse("Expenses:Food:Groceries");
        let unrelated = Account::parse("Assets:Checking");

        assert!(child.is_descendant_of(&parent));
        assert!(!parent.is_descendant_of(&child));
        assert!(!unrelated.is_descendant_of(&parent));
    }

    #[test]
    fn transaction_balanced_with_inferred_posting() {
        let date = Date::new(2026, 1, 5).unwrap();
        let mut txn = Transaction::new(date, "Grocery Store");
        txn.postings.push(Posting::new(Account::parse("Expenses:Food"), Some(usd(dec!(45.00)))));
        txn.postings.push(Posting::new(Account::parse("Assets:Checking"), None));
        assert!(txn.is_balanced());
    }

    #[test]
    fn transaction_balanced_with_explicit_amounts() {
        let date = Date::new(2026, 1, 15).unwrap();
        let mut txn = Transaction::new(date, "Salary");
        txn.postings
            .push(Posting::new(Account::parse("Assets:Checking"), Some(usd(dec!(3500.00)))));
        txn.postings
            .push(Posting::new(Account::parse("Income:Salary"), Some(usd(dec!(-3500.00)))));
        assert!(txn.is_balanced());
    }

    #[test]
    fn transaction_unbalanced_two_inferred_postings() {
        let date = Date::new(2026, 1, 1).unwrap();
        let mut txn = Transaction::new(date, "Bad entry");
        txn.postings.push(Posting::new(Account::parse("Assets:Checking"), None));
        txn.postings.push(Posting::new(Account::parse("Equity:Opening"), None));
        assert!(!txn.is_balanced());
    }
}
