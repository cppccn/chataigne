// todo, use tool in settings
// todo, dump in target.
// todo, if `path` dependencies, compute a checksum to not rebuild
// todo, if input arg `rebuild`, force build anyway
// todo, replace unwraps with error management

use crate::{
    cmd::{git::checkout_dependency, internal_run},
    common::{
        tools::{self, lib_package_path, package_paths},
        types::{Dependency, PkgFile},
    },
    settings::Settings,
};
use std::{collections::HashSet, path::PathBuf, process::Command};

/// Compile library for a static linking. Return a tuple containing a list of
/// headers (folders containing) and a list of options that the main program
/// need to compile.
pub fn compile_lib(
    dependency: &Dependency,
    pkg_file: &PkgFile,
    settings: &Settings,
) -> (HashSet<String>, HashSet<String>) {
    println!("compile lib {}", dependency.name);
    let mut headers = HashSet::new();
    let mut opts = HashSet::new();
    if let Some(lib) = &pkg_file.lib {
        let dep_path = checkout_dependency(dependency, pkg_file, settings).unwrap();
        println!("compile lib from path {}", dep_path.to_string_lossy());
        let pkg_paths = lib_package_path(lib, &dep_path).unwrap();

        for src in &pkg_paths.source_files {
            let mut cmd = Command::new("g++");
            let output = pkg_file
                .package
                .object_path(src, settings)
                .to_str()
                .unwrap()
                .to_string();
            cmd.args(vec!["-o", &output]);
            opts.insert(output);
            if let Some(opt) = &lib.opt {
                opts.extend(opt.iter().cloned())
            }
            cmd.arg("-c");
            // todo, create a diff between `headers` and `exports`
            for h in &pkg_paths.header_folders {
                let h = tools::concat(&dep_path, &h.to_string_lossy());
                cmd.arg("-I").arg(&h);
                headers.insert(h);
            }
            cmd.arg(tools::concat(&dep_path, &src.to_string_lossy()));
            internal_run(cmd);
        }
    }
    (headers, opts)
}

/// Compilation of a package given all static library `headers` dependencies
/// without links.
pub fn compile_pkg(headers: Vec<String>, test: bool) -> Vec<PathBuf> {
    let opts = headers
        .iter()
        .flat_map(|h| vec![String::from("-I"), h.clone()])
        .collect::<Vec<String>>();
    let paths = package_paths(&PathBuf::from(".")).unwrap();
    let mut objects = vec![];
    for src in &paths.source_files {
        if test
            && !src
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("test_")
        {
            continue;
        }
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
        objects.push(obj_path);
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
