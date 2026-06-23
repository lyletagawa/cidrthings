use cidrthings::{minimal_supernet, summarize_contiguous, Cidr};
use clap::Parser;
use std::io::{self, BufRead, IsTerminal};

#[derive(Parser)]
#[command(
    about = "Compute the minimal supernet that contains all given CIDR blocks",
    version
)]
struct Args {
    /// CIDR blocks to summarize (e.g. 10.1.0.0/24 10.2.0.0/24); reads from stdin if omitted
    #[arg(value_name = "CIDRs")]
    cidrs: Vec<String>,

    /// Summarize each contiguous run separately, printing one supernet per group
    #[arg(short, long)]
    summarize: bool,
}

fn main() {
    let args = Args::parse();

    let mut raw = args.cidrs;

    if !io::stdin().is_terminal() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap_or_default();
            for token in line.split([' ', '\t', ',']) {
                let t = token.trim();
                if !t.is_empty() {
                    raw.push(t.to_owned());
                }
            }
        }
    }

    if raw.is_empty() {
        eprintln!("error: no CIDR blocks provided");
        std::process::exit(1);
    }

    let mut blocks: Vec<Cidr> = Vec::with_capacity(raw.len());
    for s in &raw {
        match s.parse::<Cidr>() {
            Ok(c) => blocks.push(c),
            Err(e) => {
                eprintln!("error: {s}: {e}");
                std::process::exit(1);
            }
        }
    }

    if args.summarize {
        match summarize_contiguous(&blocks) {
            Ok(supernets) => {
                for s in supernets {
                    println!("{s}");
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        match minimal_supernet(&blocks) {
            Ok(supernet) => println!("{supernet}"),
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
