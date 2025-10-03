use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of the config file
    #[arg(short, long)]
    config: String,

    /// Launch PassDB in intercative mod
    #[arg(short, long)]
    interactive: bool,

    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Disable the output
    #[arg(short, long)]
    quiet: bool,

    /// Specify an email address to verify
    #[arg(short, long)]
    email: String,

    /// Specify an import directory
    #[arg(long)]
    import: String,

    /// Specify an output file
    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();

    println!("Hello {}!", args.config);
}
