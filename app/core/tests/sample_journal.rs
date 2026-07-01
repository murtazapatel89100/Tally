//! End-to-end check that the bundled `examples/sample.journal` parses and
//! assembles into a sensible [`Journal`] — the Phase 2 "done when" criterion.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tally_core::journal::Journal;
use tally_core::model::Account;

fn sample() -> Journal {
    // The sample lives at app/examples/sample.journal; tests run from the crate
    // dir (app/core), so climb one level up.
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/sample.journal");
    Journal::from_path(path).expect("sample.journal should parse cleanly")
}

#[test]
fn sample_journal_parses() {
    let journal = sample();

    // Nine transactions in the sample.
    assert_eq!(journal.transactions.len(), 9);

    // Declared accounts are indexed; account directives declare eight of them.
    assert!(
        journal
            .accounts
            .contains(&Account::parse("Assets:Checking"))
    );
    assert!(
        journal
            .accounts
            .contains(&Account::parse("Equity:Opening Balances"))
    );

    // No unresolved includes or unknown directives in the sample.
    assert!(
        journal.warnings.is_empty(),
        "unexpected warnings: {:?}",
        journal.warnings
    );
}

#[test]
fn every_transaction_balances() {
    for txn in &sample().transactions {
        assert!(
            txn.is_balanced(),
            "transaction on {} does not balance",
            txn.date
        );
    }
}

#[test]
fn inferred_opening_balance_is_correct() {
    let journal = sample();
    let opening = &journal.transactions[0];
    // Assets:Checking $5,000 + Assets:Savings $2,500, so the inferred
    // Equity:Opening Balances posting must be -$7,500.
    let equity = opening
        .postings
        .iter()
        .find(|p| p.account.as_str() == "Equity:Opening Balances")
        .expect("opening balance posting");
    assert_eq!(equity.amount.as_ref().unwrap().quantity, dec!(-7500.00));
}

#[test]
fn checking_balance_matches_hand_computed_total() {
    // Sum every posting that touches Assets:Checking across the file.
    let journal = sample();
    let mut total = Decimal::ZERO;
    for txn in &journal.transactions {
        for posting in &txn.postings {
            if posting.account.as_str() == "Assets:Checking" {
                total += posting.amount.as_ref().unwrap().quantity;
            }
        }
    }
    // 5000 - 123.45 + 3500 - 4.75 - 87.50 - 98.32 + 3500 - 500 = 11185.98
    assert_eq!(total, dec!(11185.98));
}
