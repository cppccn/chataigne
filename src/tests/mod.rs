use crate::{
    common::types::DepVal,
    pkg::{self},
};

#[test]
fn load_dep() {
    let pkg_file = pkg::read(Some(String::from("src/tests/dep.toml"))).unwrap();
    assert!(matches!(
        pkg_file.test.dependencies.unwrap().get("gtest").unwrap(),
        DepVal::Version(_)
    ));
    // todo: implement all other possibilities
}
