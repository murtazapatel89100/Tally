use std::path::PathBuf;

use tally_core::{
    journal::Journal,
    query::Query,
    report::{balance, register},
};

fn sample_journal() -> Journal {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/sample.journal");
    Journal::from_path(path).unwrap()
}

#[test]
fn balance_snapshot() {
    let journal = sample_journal();
    let rep = balance(&journal, &Query::default());
    insta::assert_snapshot!("balance_sample", rep.render());
}

#[test]
fn balance_assets_filter_snapshot() {
    let journal = sample_journal();
    let q = Query { account: Some("Assets".to_string()), ..Default::default() };
    let rep = balance(&journal, &q);
    insta::assert_snapshot!("balance_assets", rep.render());
}

#[test]
fn register_snapshot() {
    let journal = sample_journal();
    let rep = register(&journal, &Query::default());
    insta::assert_snapshot!("register_sample", rep.render());
}
