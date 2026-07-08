use indexmap::IndexMap;
use jiff::civil::Date;
use rust_decimal::Decimal;
use winnow::{
    ModalResult, Parser,
    ascii::space0,
    combinator::{alt, opt, preceded},
    token::{one_of, take_while},
};

use crate::{
    error::ParseError,
    model::{Account, Amount, Commodity, Posting, SourceSpan, Status, Transaction},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Transaction(Transaction),
    Directive(Directive),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Include(String),
    Alias { old: String, new: String },
    Account(String),
    Commodity(String),
    Other(String),
}

pub fn parse(input: &str, name: &str) -> Result<Vec<Entry>, ParseError> {
    parse_inner(input).map_err(|e| {
        ParseError::new(
            name,
            input.to_string(),
            (e.start, e.len),
            e.message,
            e.label,
            e.help,
        )
    })
}

struct PErr {
    start: usize,
    len: usize,
    message: String,
    label: String,
    help: Option<String>,
}

impl PErr {
    fn new(start: usize, len: usize, message: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            start,
            len,
            message: message.into(),
            label: label.into(),
            help: None,
        }
    }

    fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

fn offset_in(sub: &str, whole: &str) -> usize {
    sub.as_ptr() as usize - whole.as_ptr() as usize
}

fn parse_inner(input: &str) -> Result<Vec<Entry>, PErr> {
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut offset = 0;
    for raw in input.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\n', '\r']);
        lines.push((offset, line));
        offset += raw.len();
    }

    let mut entries = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (start, line) = lines[i];

        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        let first = line.chars().next().unwrap();
        let indented = first == ' ' || first == '\t';

        if indented {
            let content = line.trim_start();
            return Err(PErr::new(
                start + offset_in(content, line),
                content.len(),
                "posting outside of a transaction",
                "expected a transaction header before this line",
            )
            .with_help("Postings must be indented under a `DATE PAYEE` header."));
        }

        if matches!(first, ';' | '#' | '%' | '|' | '*') {
            i += 1;
            continue;
        }

        if first.is_ascii_digit() {
            let (txn, next) = parse_transaction(&lines, i)?;
            entries.push(Entry::Transaction(txn));
            i = next;
        } else {
            entries.push(Entry::Directive(parse_directive(line, start)?));
            i += 1;
            while i < lines.len() {
                let (_, l) = lines[i];
                if l.starts_with(' ') || l.starts_with('\t') {
                    i += 1;
                } else {
                    break;
                }
            }
        }
    }

    Ok(entries)
}

fn parse_transaction(lines: &[(usize, &str)], idx: usize) -> Result<(Transaction, usize), PErr> {
    let (start, header) = lines[idx];
    let mut txn = parse_header(header, start)?;
    let mut end = start + header.len();

    let mut i = idx + 1;
    while i < lines.len() {
        let (pstart, pline) = lines[i];
        if pline.trim().is_empty() {
            break;
        }
        if !(pline.starts_with(' ') || pline.starts_with('\t')) {
            break;
        }

        let content = pline.trim_start();
        if let Some(rest) = content.strip_prefix(';') {
            if let Some(last) = txn.postings.last_mut() {
                let plain = parse_comment(rest, &mut last.tags);
                append_comment(&mut last.comment, plain);
            } else {
                let plain = parse_comment(rest, &mut txn.tags);
                append_comment(&mut txn.comment, plain);
            }
        } else {
            txn.postings.push(parse_posting(pline, pstart)?);
        }

        end = pstart + pline.len();
        i += 1;
    }

    txn.source_span = Some(SourceSpan { start, end });
    infer_amounts(&mut txn);
    Ok((txn, i))
}

fn parse_header(header: &str, start: usize) -> Result<Transaction, PErr> {
    let (date_tok, rest) = header.split_once(char::is_whitespace).ok_or_else(|| {
        PErr::new(
            start,
            header.len(),
            "transaction header is missing a payee",
            "expected `DATE PAYEE` here",
        )
    })?;

    let date = parse_date(date_tok).ok_or_else(|| {
        PErr::new(
            start + offset_in(date_tok, header),
            date_tok.len(),
            "invalid date",
            "not a valid `YYYY-MM-DD` date",
        )
        .with_help("Dates look like `2026-01-15` (also `/` and `.` separators are accepted).")
    })?;

    let mut rest = rest.trim_start();

    let mut status = Status::Uncleared;
    if let Some(r) = rest.strip_prefix('*') {
        status = Status::Cleared;
        rest = r.trim_start();
    } else if let Some(r) = rest.strip_prefix('!') {
        status = Status::Pending;
        rest = r.trim_start();
    }

    let mut code = None;
    if let Some(r) = rest.strip_prefix('(')
        && let Some(close) = r.find(')')
    {
        code = Some(r[..close].to_string());
        rest = r[close + 1..].trim_start();
    }

    let (payee_part, comment_part) = match rest.split_once(';') {
        Some((p, c)) => (p, Some(c)),
        None => (rest, None),
    };

    let mut txn = Transaction::new(date, payee_part.trim());
    txn.status = status;
    txn.code = code;
    if let Some(c) = comment_part {
        let plain = parse_comment(c, &mut txn.tags);
        txn.comment = plain;
    }
    Ok(txn)
}

fn parse_posting(pline: &str, pstart: usize) -> Result<Posting, PErr> {
    let content = pline.trim_start();

    let (mut main, comment) = match content.split_once(';') {
        Some((m, c)) => (m, Some(c)),
        None => (content, None),
    };
    main = main.trim_end();

    let mut status = None;
    if let Some(r) = strip_marker(main, '*') {
        status = Some(Status::Cleared);
        main = r.trim_start();
    } else if let Some(r) = strip_marker(main, '!') {
        status = Some(Status::Pending);
        main = r.trim_start();
    }

    let (account_str, amount_str) = match find_amount_sep(main) {
        Some(sep) => (main[..sep].trim_end(), main[sep..].trim()),
        None => (main, ""),
    };

    if account_str.is_empty() {
        let at = pstart + offset_in(content, pline);
        return Err(PErr::new(
            at,
            content.len(),
            "posting is missing an account",
            "expected an account name here",
        ));
    }

    let account = Account::parse(account_str);
    let amount = if amount_str.is_empty() {
        None
    } else {
        Some(parse_amount(amount_str).ok_or_else(|| {
            let at = pstart + offset_in(amount_str, pline);
            PErr::new(at, amount_str.len(), "invalid amount", "not a valid amount")
                .with_help("Amounts look like `$1,234.56`, `-$3`, or `10 AAPL`.")
        })?)
    };

    let mut posting = Posting::new(account, amount);
    posting.status = status;
    if let Some(c) = comment {
        posting.comment = parse_comment(c, &mut posting.tags);
    }
    Ok(posting)
}

fn strip_marker(s: &str, marker: char) -> Option<&str> {
    let rest = s.strip_prefix(marker)?;
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some(rest)
    } else {
        None
    }
}

fn find_amount_sep(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\t' {
            return Some(i);
        }
        if bytes[i] == b' ' && i + 1 < bytes.len() && bytes[i + 1] == b' ' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn parse_directive(line: &str, start: usize) -> Result<Directive, PErr> {
    let (word, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let rest = rest.trim();
    match word {
        "include" => Ok(Directive::Include(rest.to_string())),
        "account" => Ok(Directive::Account(rest.to_string())),
        "commodity" => Ok(Directive::Commodity(rest.to_string())),
        "alias" => {
            let (old, new) = rest.split_once('=').ok_or_else(|| {
                PErr::new(
                    start,
                    line.len(),
                    "malformed alias directive",
                    "expected `alias OLD = NEW`",
                )
            })?;
            Ok(Directive::Alias {
                old: old.trim().to_string(),
                new: new.trim().to_string(),
            })
        }
        other => Ok(Directive::Other(other.to_string())),
    }
}

fn infer_amounts(txn: &mut Transaction) {
    if txn.postings.iter().filter(|p| p.amount.is_none()).count() != 1 {
        return;
    }
    let mut totals: IndexMap<String, (Decimal, Commodity)> = IndexMap::new();
    for p in &txn.postings {
        if let Some(a) = &p.amount {
            let entry = totals
                .entry(a.commodity.symbol.clone())
                .or_insert_with(|| (Decimal::ZERO, a.commodity.clone()));
            entry.0 += a.quantity;
        }
    }
    if totals.len() == 1 {
        let (sum, commodity) = totals.into_values().next().unwrap();
        if let Some(blank) = txn.postings.iter_mut().find(|p| p.amount.is_none()) {
            blank.amount = Some(Amount::new(-sum, commodity));
        }
    }
}

fn parse_comment(text: &str, tags: &mut IndexMap<String, Option<String>>) -> Option<String> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }

    if t.len() >= 2 && t.starts_with(':') && t.ends_with(':') {
        let inner: Vec<&str> = t[1..t.len() - 1].split(':').collect();
        if !inner.is_empty() && inner.iter().all(|s| is_tag_key(s)) {
            for name in inner {
                tags.insert(name.to_string(), None);
            }
            return None;
        }
    }

    if let Some(idx) = t.find(':') {
        let key = t[..idx].trim();
        let value = t[idx + 1..].trim();
        if is_tag_key(key) {
            tags.insert(
                key.to_string(),
                (!value.is_empty()).then(|| value.to_string()),
            );
            return None;
        }
    }

    Some(t.to_string())
}

fn is_tag_key(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn append_comment(slot: &mut Option<String>, extra: Option<String>) {
    if let Some(text) = extra {
        match slot {
            Some(existing) => {
                existing.push('\n');
                existing.push_str(&text);
            }
            None => *slot = Some(text),
        }
    }
}

pub fn parse_date(s: &str) -> Option<Date> {
    date.parse(s.trim()).ok()
}

pub fn parse_amount(s: &str) -> Option<Amount> {
    alt((prefixed_amount, suffixed_amount)).parse(s.trim()).ok()
}

fn integer(input: &mut &str) -> ModalResult<i32> {
    take_while(1.., |c: char| c.is_ascii_digit())
        .try_map(|s: &str| s.parse::<i32>())
        .parse_next(input)
}

fn date(input: &mut &str) -> ModalResult<Date> {
    (
        integer,
        one_of(['-', '/', '.']),
        integer,
        one_of(['-', '/', '.']),
        integer,
    )
        .try_map(|(y, _, m, _, d)| Date::new(y as i16, m as i8, d as i8))
        .parse_next(input)
}

fn number(input: &mut &str) -> ModalResult<Decimal> {
    take_while(1.., |c: char| c.is_ascii_digit() || c == ',' || c == '.')
        .try_map(|s: &str| Decimal::from_str_exact(&s.replace(',', "")))
        .parse_next(input)
}

fn sign(input: &mut &str) -> ModalResult<i32> {
    opt(one_of(['-', '+']))
        .map(|c| if c == Some('-') { -1 } else { 1 })
        .parse_next(input)
}

fn commodity<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    take_while(1.., |c: char| {
        !c.is_ascii_digit()
            && !c.is_whitespace()
            && c != '-'
            && c != '+'
            && c != '.'
            && c != ','
            && c != ';'
    })
    .parse_next(input)
}

fn prefixed_amount(input: &mut &str) -> ModalResult<Amount> {
    let outer = sign.parse_next(input)?;
    let sym = commodity.parse_next(input)?;
    space0.parse_next(input)?;
    let inner = sign.parse_next(input)?;
    let n = number.parse_next(input)?;
    let quantity = if outer * inner < 0 { -n } else { n };
    Ok(Amount::new(quantity, Commodity::prefixed(sym)))
}

fn suffixed_amount(input: &mut &str) -> ModalResult<Amount> {
    let s = sign.parse_next(input)?;
    let n = number.parse_next(input)?;
    let sym = opt(preceded(space0, commodity)).parse_next(input)?;
    let quantity = if s < 0 { -n } else { n };
    Ok(Amount::new(
        quantity,
        Commodity::suffixed(sym.unwrap_or("")),
    ))
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::model::CommodityPosition;

    fn only_txn(input: &str) -> Transaction {
        match parse(input, "<test>").unwrap().into_iter().next().unwrap() {
            Entry::Transaction(t) => t,
            other => panic!("expected a transaction, got {other:?}"),
        }
    }

    #[test]
    fn parses_prefixed_amount() {
        let a = parse_amount("$45.00").unwrap();
        assert_eq!(a.quantity, dec!(45.00));
        assert_eq!(a.commodity.symbol, "$");
        assert_eq!(a.commodity.position, CommodityPosition::Prefix);
    }

    #[test]
    fn parses_suffixed_amount() {
        let a = parse_amount("10 AAPL").unwrap();
        assert_eq!(a.quantity, dec!(10));
        assert_eq!(a.commodity.symbol, "AAPL");
        assert_eq!(a.commodity.position, CommodityPosition::Suffix);
    }

    #[test]
    fn parses_thousands_separator() {
        assert_eq!(parse_amount("$1,234.56").unwrap().quantity, dec!(1234.56));
    }

    #[test]
    fn parses_negatives_in_both_positions() {
        assert_eq!(parse_amount("-$3").unwrap().quantity, dec!(-3));
        assert_eq!(parse_amount("$-3").unwrap().quantity, dec!(-3));
        assert_eq!(parse_amount("-3.50 USD").unwrap().quantity, dec!(-3.50));
    }

    #[test]
    fn parses_dates_with_various_separators() {
        let d = Date::new(2026, 1, 15).unwrap();
        assert_eq!(parse_date("2026-01-15").unwrap(), d);
        assert_eq!(parse_date("2026/01/15").unwrap(), d);
        assert_eq!(parse_date("2026.01.15").unwrap(), d);
        assert!(parse_date("2026-13-01").is_none());
    }

    #[test]
    fn infers_the_blank_posting() {
        let txn = only_txn(
            "2026-01-05 * Grocery Store\n    Expenses:Food:Groceries    $123.45\n    Assets:Checking\n",
        );
        assert_eq!(txn.postings.len(), 2);
        let inferred = txn.postings[1].amount.as_ref().unwrap();
        assert_eq!(inferred.quantity, dec!(-123.45));
        assert_eq!(inferred.commodity.symbol, "$");
        assert!(txn.is_balanced());
    }

    #[test]
    fn parses_header_status_and_code() {
        let txn = only_txn("2026-01-05 * (GS-001) Grocery Store\n    A  $1\n    B  $-1\n");
        assert_eq!(txn.status, Status::Cleared);
        assert_eq!(txn.code.as_deref(), Some("GS-001"));
        assert_eq!(txn.payee, "Grocery Store");

        let pending = only_txn("2026-01-10 ! Electric\n    A  $1\n    B\n");
        assert_eq!(pending.status, Status::Pending);
    }

    #[test]
    fn keeps_single_space_account_names() {
        let txn =
            only_txn("2026-01-01 * Opening\n    Equity:Opening Balances    $5\n    Assets:Cash\n");
        assert_eq!(txn.postings[0].account.as_str(), "Equity:Opening Balances");
        assert_eq!(txn.postings[0].amount.as_ref().unwrap().quantity, dec!(5));
    }

    #[test]
    fn extracts_key_value_and_flag_tags() {
        let txn = only_txn(
            "2026-01-01 * Payee  ; project: alpha\n    A  $1  ; :reviewed:urgent:\n    B\n",
        );
        assert_eq!(txn.tags.get("project"), Some(&Some("alpha".to_string())));
        let posting_tags = &txn.postings[0].tags;
        assert!(posting_tags.contains_key("reviewed"));
        assert!(posting_tags.contains_key("urgent"));
        assert_eq!(posting_tags.get("reviewed"), Some(&None));
    }

    #[test]
    fn parses_directives() {
        let entries = parse(
            "account Assets:Checking\nalias Cash = Assets:Checking\ncommodity $\ninclude other.journal\nfoobar 1\n",
            "<test>",
        )
        .unwrap();
        assert_eq!(entries.len(), 5);
        assert_eq!(
            entries[0],
            Entry::Directive(Directive::Account("Assets:Checking".into()))
        );
        assert_eq!(
            entries[1],
            Entry::Directive(Directive::Alias {
                old: "Cash".into(),
                new: "Assets:Checking".into()
            })
        );
        assert_eq!(
            entries[2],
            Entry::Directive(Directive::Commodity("$".into()))
        );
        assert_eq!(
            entries[3],
            Entry::Directive(Directive::Include("other.journal".into()))
        );
        assert_eq!(
            entries[4],
            Entry::Directive(Directive::Other("foobar".into()))
        );
    }

    #[test]
    fn broken_amount_reports_a_caret() {
        let err = parse(
            "2026-01-01 * Bad\n    Assets:Cash    $1..2\n    B\n",
            "bad.journal",
        )
        .unwrap_err();
        assert_eq!(err.message, "invalid amount");
        let (offset, len) = (err.span.offset(), err.span.len());
        let slice = &"2026-01-01 * Bad\n    Assets:Cash    $1..2\n    B\n"[offset..offset + len];
        assert_eq!(slice, "$1..2");
    }

    #[test]
    fn source_span_covers_the_transaction() {
        let input = "2026-01-01 * Opening\n    A  $5\n    B\n\n; trailing\n";
        let txn = only_txn(input);
        let span = txn.source_span.unwrap();
        let text = &input[span.start..span.end];
        assert!(text.starts_with("2026-01-01 * Opening"));
        assert!(text.trim_end().ends_with("B"));
    }
}
