use std::process::Command;
mod gcc;
mod git;
mod new;

// todo: error on cyclic dependencies
// todo: manage pkg file, clone repo if git, checkout if commit, in any
//       case, compile if `lib` as a static lib and remember to add it in
//       the full compilation. (linking)
// todo: search all file that end with given extension (in setting and optionally in toml file)
//       respectively, add an "ignore" parameter for files or folders we want to ignore.
// todo: output should be in target/[release, debug, test]
// todo: automatically find all .h, .hpp... in the project and include all
// todo: for each file with project extension (cpp and cc are automatically taken)
// todo: find versions incompatibilities

/// Full compilation of a
pub use gcc::{compile, test};
pub use git::git_clone;
pub use new::new;
use tracing::debug;

/// Tooling, launch given command line
fn internal_run(mut cmd: Command) {
    debug!("run: {:?}", cmd);
    cmd.output().unwrap();
}
