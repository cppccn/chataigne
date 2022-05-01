// todo, use tool in settings
// todo, dump in target.
// todo, if `path` dependencies, compute a checksum to not rebuild
// todo, if input arg `rebuild`, force build anyway
// todo, replace unwraps with error management

use crate::{
    cmd::git::checkout_dependency,
    common::{
        tools::{self, find_pkg, lib_package_path, package_paths},
        types::{DepVal, Dependency, PkgFile},
    },
    settings::Settings,
};
use anyhow::Result;
use colored::Colorize;
use std::{collections::HashSet, path::PathBuf, process::Command, sync::Once};
use tracing::debug;

pub fn compile(pkg_file: PkgFile, settings: &Settings, compile_level: usize) -> Result<()> {
    let mut headers = vec![];
    let mut opts = vec![];
    let mut dependencies = pkg_file.get_dependencies(compile_level);
    debug!("Start compilation of {}", pkg_file.package.name);
    // todo: some paralelisation can be done here,
    // 1. separate git clones and compilation
    // 2. if checksums ok, compile all in paralel, otherwise check what we
    //    need to recompile
    while !dependencies.is_empty() {
        let dependency = dependencies.pop_back().unwrap();
        let pkg_file = find_pkg(&dependency, settings)?;
        let (h, o) = compile_lib(&dependency, &pkg_file, settings, compile_level);
        headers.extend(h);
        opts.extend(o);
        dependencies.extend(pkg_file.get_dependencies(compile_level));
    }

    let objects = compile_pkg(&pkg_file, headers, opts.clone(), compile_level);
    link(&pkg_file.package.name, &opts, objects);
    println!("{}", "Finishing".green());
    Ok(())
}

/// Run a compilation with the test dependencies, use the `[test]` object in
/// the settings to modify the build options, sources etc. (Nothing to do by
/// default) and launch the output.
pub fn test(pkg_file: PkgFile, settings: &Settings, compile_level: usize) -> Result<()> {
    let n = pkg_file.package.name.clone();
    compile(pkg_file, settings, compile_level)?;
    let c = Command::new(format!("./target/{n}")).spawn()?;
    c.wait_with_output().unwrap();
    Ok(())
}

/// Compile library for a static linking. Return a tuple containing a list of
/// headers (folders containing) and a list of options that the main program
/// need to compile.
pub fn compile_lib(
    dependency: &Dependency,
    pkg_file: &PkgFile,
    settings: &Settings,
    compile_level: usize,
) -> (HashSet<String>, HashSet<String>) {
    debug!("compile lib {}", dependency.name);
    let mut headers = HashSet::new();
    let mut opts = HashSet::new();
    if let Some(lib) = &pkg_file.lib {
        let dep_path = checkout_dependency(dependency, pkg_file, settings).unwrap();
        debug!("compile lib from path {}", dep_path.to_string_lossy());
        let pkg_paths =
            lib_package_path(lib, &dep_path, pkg_file.get_ignore(compile_level, true)).unwrap();

        let once = Once::new();
        for src in &pkg_paths.source_files {
            let mut cmd = Command::new("g++");
            let output = pkg_file.package.object_path(src, settings);
            let output_str = output.to_str().unwrap().to_string();
            cmd.args(vec!["-o", &output_str]);
            opts.insert(output_str);
            debug!("option {:?}", &lib.opt);
            opts.extend(lib.opt.iter().cloned());
            cmd.args(&lib.opt);
            cmd.arg("-c");
            // todo, create a diff between `headers` and `exports`
            for h in &pkg_paths.header_folders {
                let h = tools::concat(&dep_path, &h.to_string_lossy());
                cmd.arg("-I").arg(&h);
                headers.insert(h);
            }
            cmd.arg(tools::concat(&dep_path, &src.to_string_lossy()));
            match &dependency.desc {
                DepVal::Version(_) => {
                    if output.is_file() {
                        continue;
                    }
                }
                DepVal::Git(_) => todo!("git dependencies are not ready"),
                _ => {} // todo: don't compile if checksum unchanged
            };

            once.call_once(|| {
                println!(
                    "{} {} {}",
                    "Compiling".green(),
                    pkg_file.package.name,
                    pkg_file.package.version
                )
            });

            internal_run(cmd);
        }
    }
    (headers, opts)
}

/// Compilation of a package given all static library `headers` dependencies
/// without links.
pub fn compile_pkg(
    pkg_file: &PkgFile,
    headers: Vec<String>,
    mut opts: Vec<String>,
    compile_level: usize,
) -> Vec<PathBuf> {
    opts.extend(
        headers
            .iter()
            .flat_map(|h| vec![String::from("-I"), h.clone()]),
    );
    let paths = package_paths(
        &PathBuf::from("."),
        pkg_file.get_ignore(compile_level, false),
    )
    .unwrap();
    let mut objects = vec![];
    for src in &paths.source_files {
        let mut cmd = Command::new("g++");
        cmd.args(&opts);
        cmd.args(
            paths
                .header_folders
                .iter()
                .flat_map(|h| vec![String::from("-I"), h.to_string_lossy().to_string()])
                .collect::<Vec<String>>(),
        );
        let mut output = PathBuf::from("./target");
        output.push(src);
        let obj_path = output.with_extension("o");
        output.pop();
        std::fs::create_dir_all(output).unwrap();
        cmd.args(vec![
            "-c",
            &src.to_string_lossy(),
            "-o",
            &obj_path.to_string_lossy(),
        ]);
        cmd.args(&pkg_file.opt);
        objects.push(obj_path);
        // todo: don't compile if checksum unchanged
        println!(
            "{} {} {}",
            "Compiling".green(),
            pkg_file.package.name,
            pkg_file.package.version
        );
        internal_run(cmd);
    }
    objects
}

/// Latest part of he compilation is linking all dependencies, .o files and
/// shared lib links. All requirements already are field in `opts` when we
/// builded dependencies.
pub fn link(package_name: &str, opts: &Vec<String>, objects: Vec<PathBuf>) {
    let mut cmd = Command::new("g++");
    cmd.current_dir(std::env::current_dir().unwrap())
        .args(opts)
        .args(vec!["-o", &format!("target/{package_name}")])
        .args(objects.iter().map(|p| p.to_string_lossy().to_string()));
    internal_run(cmd);
}

/// Tooling, launch given command line
fn internal_run(mut cmd: Command) {
    debug!("run: {:?}", cmd);
    let c = cmd.spawn().unwrap();
    c.wait_with_output().unwrap();
    //if !c.wait_with_output().unwrap().status.success() {
    //    panic!("{}", "compile error".red().bold())
    //}
}
