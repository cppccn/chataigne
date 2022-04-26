use crate::{
    cmd::gcc::{compile_lib, compile_pkg, link},
    common::{tools::find_pkg, types::PkgFile},
    Settings,
};
use anyhow::Result;
use std::process::Command;
mod gcc;
mod git;
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
pub fn compile(pkg_file: PkgFile, settings: &Settings, compile_level: usize) -> Result<()> {
    let mut headers = vec![];
    let mut opts = vec![];
    let mut dependencies = pkg_file.get_dependencies(compile_level);
    println!("Start compilation of {}", pkg_file.package.name);
    // todo: some paralelisation can be done here
    while !dependencies.is_empty() {
        let dependency = dependencies.pop_back().unwrap();
        let pkg_file = find_pkg(&dependency, settings)?;
        let (h, o) = compile_lib(&dependency, &pkg_file, settings);
        headers.extend(h);
        opts.extend(o);
        dependencies.extend(pkg_file.get_dependencies(compile_level));
    }

    let objects = compile_pkg(headers, compile_level == 3);
    link(&pkg_file.package.name, &opts, objects);
    Ok(())
}

pub fn test(pkg_file: PkgFile, settings: &Settings, compile_level: usize) -> Result<()> {
    let n = pkg_file.package.name.clone();
    compile(pkg_file, settings, compile_level)?;
    println!("test ./target/{n}");
    let c = Command::new(format!("./target/{n}")).spawn()?;
    c.wait_with_output().unwrap();
    Ok(())
}

fn internal_run(mut cmd: Command) {
    println!("run: {:?}", cmd);
    cmd.output().unwrap();
}
