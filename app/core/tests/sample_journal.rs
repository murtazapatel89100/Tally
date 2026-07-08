//! End-to-end check that the bundled `examples/sample.journal` parses and
//! assembles into a sensible [`Journal`] — the Phase 2 "done when" criterion.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tally_core::{journal::Journal, model::Account};

fn sample() -> Journal {
    // The sample lives at app/examples/sample.journal; tests run from the crate
    // dir (app/core), so climb one level up.
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/sample.journal");
    Journal::from_path(path).expect("sample.journal should parse cleanly")
}

#[test]
fn sample_journal_parses() {
    let journal = sample();

    // 114 transactions in the sample (Jan–Jul 2026 dataset).
    assert_eq!(journal.transactions.len(), 114);

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
    // The opening entry's explicit asset and liability postings net to
    // +$11,280.00, so the single blank Equity:Opening Balances posting must be
    // inferred as -$11,280.00 to balance the transaction.
    let equity = opening
        .postings
        .iter()
        .find(|p| p.account.as_str() == "Equity:Opening Balances")
        .expect("opening balance posting");
    assert_eq!(equity.amount.as_ref().unwrap().quantity, dec!(-11280.00));
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
    // Every Assets:Checking posting across the file (explicit and inferred)
    // sums to 17137.75, verified independently against the fixture.
    assert_eq!(total, dec!(17137.75));
}
