use proptest::prelude::*;
use tally_core::{journal::Journal, printer::print_journal, query::Query, report};

fn txn_block(date: &str, payee: &str, dollars: u32) -> String {
    format!("{date} * {payee}\n    Expenses:Food    ${dollars}.00\n    Assets:Checking\n\n")
}

proptest! {
    #[test]
    fn every_parsed_txn_balances(dollars in 1u32..100_000u32) {
        let s = txn_block("2026-01-15", "Grocery Store", dollars);
        let journal = Journal::parse_str(&s).unwrap();
        for txn in &journal.transactions {
            prop_assert!(txn.is_balanced(), "transaction on {} does not balance", txn.date);
        }
    }

    #[test]
    fn parse_print_roundtrip(dollars in 1u32..100_000u32) {
        let s = txn_block("2026-03-10", "Shop", dollars);
        let j1 = Journal::parse_str(&s).unwrap();
        let printed = print_journal(&j1);
        let j2 = Journal::parse_str(&printed).unwrap();
        prop_assert_eq!(j1.transactions.len(), j2.transactions.len());
        for (t1, t2) in j1.transactions.iter().zip(j2.transactions.iter()) {
            prop_assert_eq!(&t1.payee, &t2.payee);
            prop_assert_eq!(t1.date, t2.date);
            prop_assert_eq!(t1.postings.len(), t2.postings.len());
            for (p1, p2) in t1.postings.iter().zip(t2.postings.iter()) {
                prop_assert_eq!(p1.account.as_str(), p2.account.as_str());
                prop_assert_eq!(
                    p1.amount.as_ref().map(|a| a.quantity),
                    p2.amount.as_ref().map(|a| a.quantity),
                );
            }
        }
    }

    #[test]
    fn balance_report_accounts_are_non_empty(n in 1usize..20usize) {
        let mut s = String::new();
        for i in 0..n {
            s.push_str(&txn_block(
                "2026-06-01",
                &format!("Txn {i}"),
                (i as u32 + 1) * 10,
            ));
        }
        let journal = Journal::parse_str(&s).unwrap();
        let rep = report::balance(&journal, &Query::default());
        prop_assert!(!rep.rows.is_empty(), "balance report should have rows");
    }

    #[test]
    fn register_row_count_matches_postings(n in 1usize..20usize) {
        let mut s = String::new();
        for i in 0..n {
            s.push_str(&txn_block("2026-06-01", &format!("T{i}"), (i as u32 + 1) * 5));
        }
        let journal = Journal::parse_str(&s).unwrap();
        let rep = report::register(&journal, &Query::default());
        let total_postings: usize = journal
            .transactions
            .iter()
            .map(|t| t.postings.len())
            .sum();
        prop_assert_eq!(rep.rows.len(), total_postings);
    }

    #[test]
    fn multiple_txns_parse_print_roundtrip(n in 2usize..10usize, base_dollars in 1u32..1000u32) {
        let mut s = String::new();
        for i in 0..n {
            s.push_str(&txn_block(
                "2026-09-15",
                &format!("Payee {i}"),
                base_dollars + i as u32,
            ));
        }
        let j1 = Journal::parse_str(&s).unwrap();
        let printed = print_journal(&j1);
        let j2 = Journal::parse_str(&printed).unwrap();
        prop_assert_eq!(j1.transactions.len(), j2.transactions.len());
    }
}
