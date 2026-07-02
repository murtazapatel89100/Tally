use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use miette::miette;
use tally_core::{
    journal::{Journal, JournalError},
    parser::parse_date,
    printer,
    query::Query,
    report,
};

#[derive(Parser)]
#[command(name = "tally", about = "Plain-text double-entry accounting", version)]
struct Cli {
    #[arg(short, long, env = "TALLY_FILE", global = true, help = "Journal file ($TALLY_FILE or $LEDGER_FILE)")]
    file: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Show account balances")]
    Bal {
        #[arg(help = "Account prefix filter")]
        account: Option<String>,
        #[arg(long, value_name = "DATE", help = "Start date (YYYY-MM-DD)")]
        from: Option<String>,
        #[arg(long, value_name = "DATE", help = "End date (YYYY-MM-DD)")]
        to: Option<String>,
    },
    #[command(about = "Show posting register with running total")]
    Reg {
        #[arg(help = "Account prefix filter")]
        account: Option<String>,
        #[arg(long, value_name = "DATE", help = "Start date (YYYY-MM-DD)")]
        from: Option<String>,
        #[arg(long, value_name = "DATE", help = "End date (YYYY-MM-DD)")]
        to: Option<String>,
    },
    #[command(about = "List all known accounts")]
    Accounts,
    #[command(about = "Print journal in canonical form")]
    Print,
    #[command(hide = true, about = "Print shell completions")]
    Completions { shell: Shell },
}

fn main() -> miette::Result<()> {
    color_eyre::install().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    let file = cli
        .file
        .or_else(|| std::env::var("LEDGER_FILE").ok().map(PathBuf::from));

    match cli.command {
        Command::Accounts => {
            let journal = load(file)?;
            for account in &journal.accounts {
                println!("{}", account.as_str());
            }
        }

        Command::Print => {
            let journal = load(file)?;
            print!("{}", printer::print_journal(&journal));
        }

        Command::Bal { account, from, to } => {
            let journal = load(file)?;
            let query = build_query(account, from, to)?;
            let rep = report::balance(&journal, &query);
            print!("{}", rep.render());
        }

        Command::Reg { account, from, to } => {
            let journal = load(file)?;
            let query = build_query(account, from, to)?;
            let rep = report::register(&journal, &query);
            print!("{}", rep.render());
        }

        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "tally", &mut std::io::stdout());
        }
    }

    Ok(())
}

fn load(file: Option<PathBuf>) -> miette::Result<Journal> {
    let path =
        file.ok_or_else(|| miette!("no journal file; use -f FILE or set $TALLY_FILE"))?;
    Journal::from_path(&path).map_err(|e| match e {
        JournalError::Parse(pe) => miette::Report::new(pe),
        JournalError::Io { path, source } => miette!("cannot read '{path}': {source}"),
    })
}

fn build_query(
    account: Option<String>,
    from: Option<String>,
    to: Option<String>,
) -> miette::Result<Query> {
    let from = from
        .map(|s| parse_date(&s).ok_or_else(|| miette!("invalid date '{s}'")))
        .transpose()?;
    let to = to
        .map(|s| parse_date(&s).ok_or_else(|| miette!("invalid date '{s}'")))
        .transpose()?;
    Ok(Query { account, from, to, ..Default::default() })
}
