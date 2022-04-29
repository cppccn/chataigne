use clap::StructOpt;
use cli::{Cli, Commands};
use cmd::compile;
use settings::Settings;

mod cli;
mod cmd;
mod common;
mod pkg;
mod settings;

const DEFAULT_PACKAGE_FILE_NAME: &str = "chataigne.toml";

#[cfg(test)]
mod tests;

fn main() {
    // todo: give a directory to the luc.toml folder container or to the toml
    //       directly in parameter
    // todo: take second argument "build", "warn", "fmt", install.
    // todo: if unknown argument, search in ProjectDirs/bin the binary to call
    // todo: nice unwrap handling
    let settings = Settings::new().unwrap();
    let cli: &Cli = &cli::Cli::parse();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(cli.verbosity())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Build(cmd) => {
                let root_pkg_file = pkg::read(Some(DEFAULT_PACKAGE_FILE_NAME.to_string())).unwrap();
                let level = cmd.compilation_level();
                compile(root_pkg_file, &settings, level).unwrap();
            }
            Commands::New { name } => cmd::new(name),
            Commands::Test => {
                let root_pkg_file = pkg::read(Some(DEFAULT_PACKAGE_FILE_NAME.to_string())).unwrap();
                cmd::test(root_pkg_file, &settings, 3).unwrap();
            }
        };
    }
}
