//! Usage of clap to parse cli parameters
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
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
