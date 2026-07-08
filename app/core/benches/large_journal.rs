use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tally_core::journal::Journal;
use tally_core::query::Query;
use tally_core::report;

fn generate_journal(n: usize) -> String {
    let mut out = String::new();
    for i in 0..n {
        let day = (i % 28 + 1) as u8;
        let month = (i / 28 % 12 + 1) as u8;
        let year = 2020 + (i / 336) as u16;
        let dollars = i * 7 + 1;
        out.push_str(&format!(
            "{year}-{month:02}-{day:02} * Transaction {i}\n    Expenses:Food    ${dollars}.00\n    Assets:Checking\n\n"
        ));
    }
    out
}

fn bench_parse(c: &mut Criterion) {
    let mut g = c.benchmark_group("parse");
    for n in [100, 500, 1_000, 5_000] {
        let input = generate_journal(n);
        g.bench_with_input(BenchmarkId::from_parameter(n), &input, |b, s| {
            b.iter(|| Journal::parse_str(black_box(s)).unwrap())
        });
    }
    g.finish();
}

fn bench_balance_report(c: &mut Criterion) {
    let mut g = c.benchmark_group("balance_report");
    for n in [100, 1_000] {
        let input = generate_journal(n);
        let journal = Journal::parse_str(&input).unwrap();
        g.bench_with_input(BenchmarkId::from_parameter(n), &journal, |b, j| {
            b.iter(|| report::balance(black_box(j), &Query::default()))
        });
    }
    g.finish();
}

fn bench_register_report(c: &mut Criterion) {
    let mut g = c.benchmark_group("register_report");
    for n in [100, 1_000] {
        let input = generate_journal(n);
        let journal = Journal::parse_str(&input).unwrap();
        g.bench_with_input(BenchmarkId::from_parameter(n), &journal, |b, j| {
            b.iter(|| report::register(black_box(j), &Query::default()))
        });
    }
    g.finish();
}

fn bench_print(c: &mut Criterion) {
    let mut g = c.benchmark_group("print");
    for n in [100, 1_000] {
        let input = generate_journal(n);
        let journal = Journal::parse_str(&input).unwrap();
        g.bench_with_input(BenchmarkId::from_parameter(n), &journal, |b, j| {
            b.iter(|| tally_core::printer::print_journal(black_box(j)))
        });
    }
    g.finish();
}

criterion_group!(benches, bench_parse, bench_balance_report, bench_register_report, bench_print);
criterion_main!(benches);
