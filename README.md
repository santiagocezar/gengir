# GenGIR: Genuine* autocompletion for your PyGObject code!

This tool initially started as a fork of [fakegir](https://github.com/strycore/fakegir) but now it has been rewritten entirely.

GenGIR is a tool to create type definitions for PyGObject. It uses modern python standards and it's easy to use 

## Features

- Supports [PEP 484](https://www.python.org/dev/peps/pep-0484/) type annotations
- Installs typings as a [PEP 561](https://www.python.org/dev/peps/pep-0561/) stub in the correct site-packages, even for venv!
  It creates a package named `gi-stubs`. Once it's installed, it should be recognized by your IDE and it should provide autocompletion and typing errors.
- Uses Sphinx markup on docstrings
- A GTK version switch 
- Multithreading!

## TODO

- Complete [`overrides.rs`](src/overrides.rs)
- Correct typings for constructors
- Typings for `.connect` signal names and callbacks

## Building & Installing

To build this project, you need to have installed the Rust toolchain, version 1.56.0 or newer.

`git clone` this repository, and run `cargo build --release`.

You can run the program once using `cargo run --release`, but if you use separate `venv`s for your projects, I'd recommend installing it user wide with `cargo install --path .`


## Usage

The `*.gir` with the type info files should be included with each GNOME library development package in `/usr/share/gir-1.0/`.

If you want to install the stub user wide, just run gengir. If you're using a venv you'll need to run gengir inside the venv. With poetry for example just run `poetry run gengir`.

```
USAGE:
    gengir [OPTIONS] [GIR_FILES]...

ARGS:
    <GIR_FILES>...    Files to use as input for the generator. If not provided it uses all files
                      in /usr/share/gir-1.0/

OPTIONS:
        --gtk <GTK_VERSION>    GTK version to generate typings for [default: 3]
    -h, --help                 Print help information
    -n, --no-docs              Exclude docstrings in the typings
    -o, --out-dir <OUT_DIR>    
    -V, --version              Print version information

```

## Editor support

-   VSCode has support for stub packages out of the box.
-   [Jedi](https://github.com/davidhalter/jedi) supports it too, so any editor using it should work.
