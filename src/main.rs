use clap::StructOpt;
use cli::Commands;
use cmd::compile;
use settings::Settings;

mod cli;
mod cmd;
mod common;
mod pkg;
mod settings;

#[cfg(test)]
mod tests;

fn main() {
    // todo: give a directory to the luc.toml folder container or to the toml
    //       directly in parameter
    // todo: take second argument "build", "warn", "fmt", install.
    // todo: if unknown argument, search in ProjectDirs/bin the binary to call
    // todo: nice unwrap handling
    // todo, replace println with tracing. use verbosity as argument / settings
    let settings = Settings::new().unwrap();
    let cli = &cli::Cli::parse();
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Build(cmd) => {
                let root_pkg_file = pkg::read(Some("luc.toml".to_string())).unwrap();
                let level = cmd.compilation_level();
                compile(root_pkg_file, &settings, level).unwrap();
            }
            Commands::New { name } => {
                todo!("generate {name} hello world")
            }
            Commands::Test => {
                let root_pkg_file = pkg::read(Some("luc.toml".to_string())).unwrap();
                cmd::test(root_pkg_file, &settings, 3).unwrap();
            }
            _ => todo!(),
        };
    }
}
