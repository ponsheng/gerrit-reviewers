use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   /// Name of the person to greet
   #[arg(short, long)]
   pub url: Option<String>,

   #[arg(short, long)]
   pub change: Option<String>,

   #[clap(flatten)]
   pub verbose: clap_verbosity_flag::Verbosity,
}

pub fn parse() -> Args {
    let args = Args::parse();
    
    if let Some(s) = &args.url {
        println!("url: {}", s);
    }

    if let Some(s) = &args.change {
        println!("change: {}", s);
    }

    // TODO semantic check

    args
}

