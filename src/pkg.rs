use crate::common::types::{
    BuildOption, ConfigBuildOption, ConfigPackage, ConfigPkgDescription, DepVal, Package,
    PkgDescription, SrcVal,
};
use anyhow::{bail, Result};

// todo: put all `deserializable` things into a sub package dedicated.
//       - PkgFileInternal
//       - LocalDependency
//       - GitDependency

impl TryFrom<ConfigPkgDescription> for PkgDescription {
    type Error = anyhow::Error;
    fn try_from(p: ConfigPkgDescription) -> Result<Self, Self::Error> {
        let src = match p.src {
            Some(t) => {
                if let Ok(git) = t.clone().try_into() {
                    Some(SrcVal::Git(git))
                } else if let Ok(local) = t.try_into() {
                    Some(SrcVal::Local(local))
                } else {
                    bail!("unexpected src value")
                }
            }
            None => None,
        };
        Ok(Self {
            name: p.name,
            version: p.version,
            description: p.description,
            src,
            repostory: p.repostory,
        })
    }
}

impl From<ConfigBuildOption> for BuildOption {
    fn from(b: ConfigBuildOption) -> Self {
        Self {
            ignore: b.ignore,
            dependencies: DepVal::adapt(b.dependencies),
            sources: b.sources,
            includes: b.includes,
            opt: b.opt,
        }
    }
}

impl From<ConfigPackage> for Package {
    fn from(mut i: ConfigPackage) -> Self {
        if i.dev.ignore.is_empty() {
            i.dev.ignore.push(String::from("**/test.cpp"));
        }
        if i.ignore.is_empty() {
            i.ignore.push(String::from("**/test.cpp"));
        }
        if i.test.ignore.is_empty() {
            i.test.ignore.push(String::from("**/main.cpp"));
        }
        Self {
            dependencies: DepVal::adapt(i.dependencies),
            dev: i.dev.into(),
            test: i.test.into(),
            ignore: i.ignore,
            lib: i.lib,
            opt: i.opt,
            sources: i.package.sources.clone(),
            includes: i.package.includes.clone(),
            pkg_description: i.package.try_into().unwrap(),
        }
    }
}
