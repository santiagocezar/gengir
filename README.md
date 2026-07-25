# GenGIR: Genuine* autocompletion for your PyGObject code!

> ⚠️ [This project](https://github.com/santiagocezar/gengir/) was rewritten to Rust a while ago, and is not receiving any updates.

[![PyPI version](https://badge.fury.io/py/gengir.svg)](https://pypi.org/project/gengir/)

[fakegir](https://github.com/strycore/fakegir) is a tool to build a fake python package of PyGObject modules.

GenGIR is a fork of fakegir that uses modern standards and improves on usability 

The main changes are:

-   Use [PEP 484](https://www.python.org/dev/peps/pep-0484/) type annotations instead of docstrings
-   Install typings as a [PEP 561](https://www.python.org/dev/peps/pep-0561/) stub
-   Use Sphinx markup on docstrings
-   Fields are typed now
-   A cli

GenGIR stores the type info inside your user or venv site-packages as a package named `gi-stubs`.  
Once it's installed, it should be recognized by your IDE and it should provide autocompletion and typing errors.

## Installation

gengir is available on [pypi](https://pypi.org/project/gengir/), install it using

```
$ pip install gengir
```

or add it as a development dependency if you're using poetry

## TODO

- Documentation formatting (code snippets, links)
- Signals as `Literal`s

## Building

This project uses [poetry](https://python-poetry.org/), so make sure to have that installed

Then run `poetry install` and `poetry build`. A wheel file should be created in the dist directory that can be installed using `pip install ./dist/gengir*.whl`


## Usage

The `*.gir` with the type info files should be included with each GNOME library development package in `/usr/share/gir-1.0/`.

```
gengir [OPTIONS] [TYPES]...

  Generate PEP 561 stubs for the GObject introspection library.

Arguments:
  [TYPES]...  Files to use as input for the generator. If not provided it uses
              all files in /usr/share/gir-1.0/

Options:
  -o, --outdir PATH               Directory to store the package typings.
                                  $site-packages/gi-stubs by default
  --docs / --no-docs              Include docstrings in the typings  [default:
                                  docs]
  --gtk INTEGER                   GTK version to generate typings for
                                  [default: 3]
```

## Editor support

-   VSCode has support for stub packages out of the box.
-   [Jedi](https://github.com/davidhalter/jedi) supports it too, so any editor using it should work.
