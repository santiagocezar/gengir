#!/usr/bin/env python3
"""Build a fake python package from the information found in gir files"""
import glob
import os
from pathlib import Path
import multiprocessing
import logging
from typing import Optional, TypedDict
import astor

from lxml.etree import XML, XMLParser

from gengir.generate import StubGenerator

logger = logging.getLogger(__name__)

XMLNS = "http://www.gtk.org/introspection/core/1.0"


class Options(TypedDict):
    files: list[Path]
    outdir: Path
    include_docs: bool
    gtk_version: int


GIR_PATH = Path("/usr/share/gir-1.0")


def iter_girs(opt: Options):
    files = opt["files"]
    gtk_version = opt["gtk_version"]

    """Return a generator of all available gir files"""
    gir_files = files if len(files) > 0 else GIR_PATH.glob("*.gir")

    for gir_file in gir_files:
        basename = gir_file.name

        if len(files) == 0:
            # Check which GTK Version the user wants to use
            if (
                (
                    basename in ("Gtk-2.0.gir", "Gdk-2.0.gir", "GdkX11-2.0.gir")
                    and gtk_version != 2
                )
                or (
                    basename in ("Gtk-3.0.gir", "Gdk-3.0.gir", "GdkX11-3.0.gir")
                    and gtk_version != 3
                )
                or (
                    basename in ("Gtk-4.0.gir", "Gdk-4.0.gir", "GdkX11-4.0.gir")
                    and gtk_version != 4
                )
            ):
                continue

        try:
            module_name = basename[: basename.index("-")]
        except ValueError:
            # file name contains no dashes
            logger.warning(f"Warning: unrecognized file in gir directory: {gir_file}")
            continue

        yield (opt, module_name, gir_file)


def generate_module(gir_info: tuple[Options, str, Path]):
    opt, module_name, gir_path = gir_info

    print(f"Parsing {gir_path}")
    parser = XMLParser(encoding="utf-8", recover=True)
    with open(gir_path, "rt", encoding="utf-8") as gir_file:
        content = gir_file.read()
    root = XML(content, parser)

    namespace = root.find("namespace", root.nsmap)

    if namespace is None:
        print(f"{gir_path} had no root namespace, weird...")
        return

    gen = StubGenerator(include_docs=opt["include_docs"])

    namespace_content = astor.to_source(gen.extract_namespace(namespace))

    fakegir_path = opt["outdir"] / "repository" / (module_name + ".pyi")

    with open(fakegir_path, "w") as fakegir_file:
        fakegir_file.write("# -*- coding: utf-8 -*-\n")
        fakegir_file.write(namespace_content)


def generate_fakegir(opt: Options):
    """Main function"""

    os.makedirs(opt["outdir"] / "repository", exist_ok=True)

    with open(opt["outdir"] / "__init__.pyi", "w") as init:
        init.write("def require_version(namespace, version):\n")
        init.write("    pass\n")

    with open(opt["outdir"] / "py.typed", "w") as py_typed:
        py_typed.write("partial\n")

    with open(opt["outdir"] / "repository/__init__.pyi", "w") as repo_init:
        repo_init.write("")

    pool = multiprocessing.Pool()

    pool.map(generate_module, iter_girs(opt))
