use rust_decimal::Decimal;

use crate::journal::Journal;
use crate::model::{Amount, CommodityPosition, Transaction};

pub fn print_journal(journal: &Journal) -> String {
    let mut out = String::new();
    for txn in &journal.transactions {
        out.push_str(&print_transaction(txn));
        out.push('\n');
    }
    out
}

pub fn print_transaction(txn: &Transaction) -> String {
    let mut out = String::new();

    out.push_str(&txn.date.to_string());
    if let Some(marker) = txn.status.marker() {
        out.push(' ');
        out.push(marker);
    }
    if let Some(ref code) = txn.code {
        out.push_str(&format!(" ({code})"));
    }
    out.push(' ');
    out.push_str(&txn.payee);
    if let Some(ref comment) = txn.comment {
        out.push_str(&format!("  ; {comment}"));
    }
    out.push('\n');

    for posting in &txn.postings {
        let acc = posting.account.as_str();
        match &posting.amount {
            None => out.push_str(&format!("    {acc}")),
            Some(a) => {
                let amt = format_amount(a);
                let header = format!("    {acc}");
                let pad_to = 48usize;
                let padding = if header.len() + 2 < pad_to {
                    " ".repeat(pad_to - header.len())
                } else {
                    "  ".to_string()
                };
                out.push_str(&format!("{header}{padding}{amt}"));
            }
        }
        if let Some(ref comment) = posting.comment {
            out.push_str(&format!("  ; {comment}"));
        }
        out.push('\n');
    }

    out
}

pub fn format_amount(amount: &Amount) -> String {
    let qty = amount.quantity;
    let abs_str = format_number(qty.abs());
    match amount.commodity.position {
        CommodityPosition::Prefix => {
            if qty < Decimal::ZERO {
                format!("-{}{}", amount.commodity.symbol, abs_str)
            } else {
                format!("{}{}", amount.commodity.symbol, abs_str)
            }
        }
        CommodityPosition::Suffix => {
            let sign = if qty < Decimal::ZERO { "-" } else { "" };
            if amount.commodity.symbol.is_empty() {
                format!("{sign}{abs_str}")
            } else {
                format!("{sign}{abs_str} {}", amount.commodity.symbol)
            }
        }
    }
}

pub fn format_number(qty: Decimal) -> String {
    let s = qty.to_string();
    let (int_part, dec_part) = match s.find('.') {
        Some(pos) => (&s[..pos], &s[pos..]),
        None => (s.as_str(), ""),
    };
    let chars: Vec<char> = int_part.chars().collect();
    let len = chars.len();
    let mut result = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result + dec_part
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::model::Commodity;

    fn usd(q: rust_decimal::Decimal) -> Amount {
        Amount::new(q, Commodity::prefixed("$"))
    }

    #[test]
    fn formats_prefixed_positive() {
        assert_eq!(format_amount(&usd(dec!(45.00))), "$45.00");
    }

    #[test]
    fn formats_prefixed_negative() {
        assert_eq!(format_amount(&usd(dec!(-3.00))), "-$3.00");
    }

    #[test]
    fn formats_thousands() {
        assert_eq!(format_amount(&usd(dec!(1234.56))), "$1,234.56");
        assert_eq!(format_amount(&usd(dec!(1000000))), "$1,000,000");
    }

    #[test]
    fn formats_suffixed() {
        let a = Amount::new(dec!(10), Commodity::suffixed("AAPL"));
        assert_eq!(format_amount(&a), "10 AAPL");
        let b = Amount::new(dec!(-5), Commodity::suffixed("AAPL"));
        assert_eq!(format_amount(&b), "-5 AAPL");
    }

    #[test]
    fn roundtrip_via_parse() {
        use crate::journal::Journal;
        use crate::model::Transaction;

        fn strip_spans(txns: Vec<Transaction>) -> Vec<Transaction> {
            txns.into_iter().map(|mut t| { t.source_span = None; t }).collect()
        }

        let journal = Journal::parse_str(
            "2026-01-05 * Grocery Store\n    Expenses:Food:Groceries    $123.45\n    Assets:Checking\n",
        )
        .unwrap();
        let printed = print_journal(&journal);
        let journal2 = Journal::parse_str(&printed).unwrap();
        assert_eq!(strip_spans(journal.transactions), strip_spans(journal2.transactions));
    }
}
