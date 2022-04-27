//! Usage of clap to parse cli parameters
use clap::{Parser, Subcommand};
use tracing::Level;
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
    #[clap(short, long, default_value_t = 0)]
    verbose: u8,
}

impl Cli {
    pub fn verbosity(&self) -> Level {
        if self.verbose > 5 {
            return Level::TRACE;
        }
        match 4 - self.verbose {
            1 => Level::DEBUG,
            2 => Level::INFO,
            3 => Level::WARN,
            4 => Level::ERROR,
            _ => Level::TRACE,
        }
    }
}

impl BuildSubCommand {
    pub fn compilation_level(&self) -> usize {
        if self.release {
            1
        } else if self.test {
            3
        } else {
            2
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    Build(BuildSubCommand),
    Clean,
    Clear,
    Check,
    Test,
    Fmt,
    New { name: String },
}

#[derive(clap::Args)]
pub struct BuildSubCommand {
    #[clap(short, long)]
    pub release: bool,
    #[clap(short, long)]
    pub test: bool,
    #[clap(short, long)]
    pub lib: bool,
    #[clap(short, long)]
    pub shared: bool,
}
