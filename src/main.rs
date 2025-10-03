use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of the config file
    #[arg(short, long, default_value = "passdb.yml")]
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
    #[arg(long, default_value = "import/")]
    import: String,

    /// Specify an output file
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let mut args = Args::parse();

    if args.output.is_none() {
        args.output = Some(format!("{}.txt", args.email));
    }
    println!("Email {}", args.email);
    if let Some(output) = &args.output {
        println!("Output file {}", output);
    } else {
        println!("No output file");
    }
}
