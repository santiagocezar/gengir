from pathlib import Path
from typing import Optional
import typer
import os
import sys
import site

from gengir.main import generate_fakegir


def cli(
    types: Optional[list[Path]] = typer.Argument(
        None,
        help="Files to use as input for the generator. If not provided it uses all files in /usr/share/gir-1.0/",
    ),
    outdir: Optional[Path] = typer.Option(
        None,
        "--outdir",
        "-o",
        help="Directory to store the package typings. $site-packages/gi-stubs by default",
    ),
    docs: bool = typer.Option(True, help="Include docstrings in the typings"),
    gtk: int = typer.Option(3, help="GTK version to generate typings for"),
):
    """
    Generate PEP 561 stubs for the GObject introspection library.
    """

    sp = Path(site.getsitepackages()[0])
    if not os.access(sp, os.W_OK | os.X_OK):
        user_sp = Path(site.getusersitepackages())
        print(f"Can't write to {sp}, using {user_sp} instead", file=sys.stderr)
        sp = user_sp

    outdir = outdir or sp / "gi-stubs"

    generate_fakegir(
        {
            "files": types or [],
            "gtk_version": gtk,
            "include_docs": docs,
            "outdir": outdir,
        }
    )


def run_cli():
    typer.run(cli)
