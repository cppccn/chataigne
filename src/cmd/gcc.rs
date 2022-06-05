// todo, use tool in settings
// todo, dump in target.
// todo, if `path` dependencies, compute a checksum to not rebuild
// todo, if input arg `rebuild`, force build anyway
// todo, replace unwraps with error management
use crate::{
    cmd::git::checkout_dependency,
    common::{
        tools::{self, find_pkg},
        types::{DepVal, Dependency, Package},
    },
    settings::Settings,
};
use anyhow::Result;
use colored::Colorize;
use std::{collections::HashSet, path::PathBuf, process::Command, sync::Once};
use tracing::debug;

pub fn compile(package: &Package, settings: &Settings, compile_level: usize) -> Result<()> {
    let mut headers = vec![];
    let mut opts = package.get_opt(compile_level);
    let mut dependencies = package.get_dependencies(compile_level);
    debug!("Start compilation of {}", package.pkg_description.name);
    debug!("Package options {:?}", opts);
    // todo: some paralelisation can be done here,
    // 1. separate git clones and compilation
    // 2. if checksums ok, compile all in paralel, otherwise check what we
    //    need to recompile
    while !dependencies.is_empty() {
        let dependency = dependencies.pop_back().unwrap();
        let pkg_file = find_pkg(&dependency, settings)?;
        let (h, o) = compile_lib(&dependency, &pkg_file, settings)?;
        headers.extend(h);
        opts.extend(o);
        dependencies.extend(pkg_file.get_dependencies(compile_level));
    }

    let objects = compile_pkg(package, headers, opts.clone(), compile_level)?;
    link(&package.pkg_description.name, &opts, objects);
    println!("{}", "Finishing".green());
    Ok(())
}

/// Run a compilation with the test dependencies, use the `[test]` object in
/// the settings to modify the build options, sources etc. (Nothing to do by
/// default) and launch the output.
pub fn test(package: &Package, settings: &Settings) -> Result<()> {
    let n = package.pkg_description.name.clone();
    compile(package, settings, 3)?;
    let c = Command::new(format!("./target/{n}")).spawn()?;
    c.wait_with_output()?;
    Ok(())
}

/// Compile library for a static linking. Return a tuple containing a list of
/// headers (folders containing) and a list of options that the main program
/// need to compile.
pub fn compile_lib(
    dependency: &Dependency,
    package: &Package,
    settings: &Settings,
) -> Result<(HashSet<String>, HashSet<String>)> {
    debug!("compile lib {}", dependency.name);
    let mut headers = HashSet::new();
    let mut opts = HashSet::new();
    let lib = package.get_lib()?;
    let dep_path = checkout_dependency(dependency, package, settings)?;
    debug!("compile lib from path {}", dep_path.to_string_lossy());
    let pkg_paths = package.lib_package_path(&dep_path)?;

    let once = Once::new();
    for src in &pkg_paths.source_files {
        let mut cmd = Command::new("gcc");
        let output = package.object_path(src, settings);
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
                package.pkg_description.name,
                package.pkg_description.version
            )
        });

        internal_run(cmd);
    }
    Ok((headers, opts))
}

/// Compilation of a package given all static library `headers` dependencies
/// without links.
///
/// Take in input the `headers` (what we need to include) that had been deduced
/// when the dependencies has been build. We also give the `opts`
pub fn compile_pkg(
    package: &Package,
    headers: Vec<String>,
    mut opts: Vec<String>,
    compile_level: usize,
) -> Result<Vec<PathBuf>> {
    opts.extend(
        headers
            .iter()
            .flat_map(|h| vec![String::from("-I"), h.clone()]),
    );
    let paths = package.root_package_paths(compile_level)?;
    let mut objects = vec![];
    let opts = package.get_opt(compile_level);
    for src in &paths.source_files {
        let mut cmd = Command::new("gcc");
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
        cmd.args(&opts);
        objects.push(obj_path);
        // todo: don't compile if checksum unchanged
        println!(
            "{} {} {}",
            "Compiling".green(),
            package.pkg_description.name,
            package.pkg_description.version,
        );
        internal_run(cmd);
    }
    Ok(objects)
}

/// Latest part of he compilation is linking all dependencies, .o files and
/// shared lib links. All requirements already are field in `opts` when we
/// builded dependencies.
pub fn link(package_name: &str, opts: &Vec<String>, objects: Vec<PathBuf>) {
    let mut cmd = Command::new("gcc");
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
