use std::{path::PathBuf, process::Command};

use crate::DEFAULT_PACKAGE_FILE_NAME;

const HELLO_WORLD: &str = r#"#include <iostream>

int main() {
	std::cout << "Hello World!";
	return 0;
}
"#;

pub fn new(package_name: &str) {
    Command::new("git")
        .arg("init")
        .arg(package_name)
        .output()
        .unwrap();
    let mut p = PathBuf::from(package_name);
    let mut main = p.clone();
    main.push("main.cpp");
    std::fs::write(main, HELLO_WORLD).unwrap();
    p.push(DEFAULT_PACKAGE_FILE_NAME);
    std::fs::write(
        p,
        format!(
            r#"[package]
name="{}"
version="0.1.0"
"#,
            package_name
        ),
    )
    .unwrap();
}
