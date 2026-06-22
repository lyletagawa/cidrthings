use cidrthings::{minimal_supernet, Cidr};
use clap::Parser;

#[derive(Parser)]
#[command(
    about = "Compute the minimal supernet that contains all given CIDR blocks",
    version
)]
struct Args {
    /// CIDR blocks to summarize (e.g. 10.1.0.0/24 10.2.0.0/24)
    #[arg(required = true)]
    cidrs: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let mut blocks: Vec<Cidr> = Vec::with_capacity(args.cidrs.len());
    for s in &args.cidrs {
        match s.parse::<Cidr>() {
            Ok(c) => blocks.push(c),
            Err(e) => {
                eprintln!("error: {s}: {e}");
                std::process::exit(1);
            }
        }
    }

    match minimal_supernet(&blocks) {
        Ok(supernet) => println!("{supernet}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
