# Chataigne 🌰

The project is actually a _Minimum Viable Product_ that we can use from basics to big projects.

However, Chataigne is a _gcc_-based dependency mangement that extract the
complexity of writing specific CMakeList, Makefile, with a fixed and simple
language that forbade users writing frameworks onthere and profits of the
community developments to work.

## Installation

### Get binary

You can download the latest release on this repository or if you're using Rust,
you probably would like to build a binary from sources.

Generate the executable `ch` in `./chataigne/target/release/`
```bash
git clone https://github.com/cppccn/chataigne.git
cd chataigne
cargo build --release
```

You will need to add the binary to your environment variable or add the binary
into a bin folder like `/home/username/.nix-profile/bin/` if you use nix.

Here is a tutorial: [add env path in linux](https://www.baeldung.com/linux/path-variable).

## Get started

Chataigne use the short command line `ch` that accept multiple parameters.
Use `--help` for a quick view of possibilities.

Let's create a new folder with a chataigne configuration:

```bash
ch new hello
cd hello
ls
#chataigne.toml  main.cpp
```

You get the minimal c++ project and you can add dependencies as in cook book
modifying the `chataigne.toml` file.

```toml
# Example of a chataigne file that use googletest

[package]
name="hello"
version="0.1.0"
ignore=["test.cpp"]
# Ignoring is an optional precision, any files named `test.cpp` are ignored by
# default. However, you can override that behavior.

[dependencies]
my_lib={ path = "../my_lib_cpp" }
# Import my own local library in release, dev and test

[dev.dependencies]
my_lib={ path = "../my_lib_cpp_dev" }
# You can import other dependencies but also override the release version in
# dev mode. It can be useful to test new features.

[test.dependencies]
gtest="1.11.0"
# Import gtest library from web, you just have to precise the version. Any
# library defined in layers can be loaded that way.

[test]
ignore=["main.cpp"]
# Same as the global ignore, any `main.cpp` is ignored by default in test mode.
# The `main()` function is considered declared in a `test.cpp`

```

The command `ch build` generate a target folder with `{package.name}`
executable. `ch run` will generate and run the target.

By default, the `dev` version is generated with the chataigne build. If you
need to build or run the release version, use flags `--release` or `--dev`.

## Roadmap

List of features that we will add:

- Load and compile dependencies asynchronously.
- Fix the local compilation management.
  - Reprocess compilation only if code changed.
  - Fix naming in cache.
- Add cache commands.
  - List cache packages.
  - Remove and add packages.
  - Force rebuild.
  - List path of project's dependencies to simplify 
- Improve error management.
  - Print gcc output in case of error.
  - Handle errors correctly with good outputs.
- Improve release.
  - Use optimization builds flags with gcc.
- Library load.
  - Load a library directly from a git repository.
  - Instead of loading libraries and building output, generate a default.nix
  - add a "pre-builded" option
  - add a "shared" option (for both dll and so)
- Sources
  - Actually headers and sources are by default `.h` and `.cpp`, it will be
    more flexible.
- User feature
  - Add a `--boilerplate` flag when creating a new project that dump a minimal
    project.
- Creation of a website with a documentation for layers, settings and chataigne files

## Contribute

Look at the [contibute.md](CONTRIBUTE.md)

## License

That project is under an MIT license.

<small>
Notes about development:

Even if we promise that we will not do breaking changes, there is no guaranty
that future versions will not introduce bugs. We'll probably in a few
month/years set a kind of stable version. We explain that because we'll try to keep
flexibility about and accept many contributions on that project, we want to use many
dependencies (following the same ideologie of what we want to introduce with
chataigne).
</small>