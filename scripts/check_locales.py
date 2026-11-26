"""Confere as traduções em relação ao catálogo inglês (Python 3.11+)."""

from collections import Counter
from pathlib import Path
import re
import tomllib


LOCALES = Path(__file__).resolve().parents[1] / "locales"
PLACEHOLDER = re.compile(r"%\{\w+\}")


def read_catalog(path):
    with path.open("rb") as source:
        return tomllib.load(source)


def check_catalog(path, source):
    catalog = read_catalog(path)
    errors = []
    for key in sorted(source.keys() - catalog.keys()):
        errors.append(f"chave faltante: {key}")
    for key in sorted(catalog.keys() - source.keys()):
        errors.append(f"chave desconhecida: {key}")
    for key in sorted(source.keys() & catalog.keys()):
        message = catalog[key]
        if not isinstance(message, str) or not message.strip():
            errors.append(f"texto inválido: {key}")
        elif Counter(PLACEHOLDER.findall(message)) != Counter(PLACEHOLDER.findall(source[key])):
            errors.append(f"placeholders diferentes do inglês: {key}")
    for error in errors:
        print(f"{path.name}: {error}")
    return not errors


def main():
    source = read_catalog(LOCALES / "en.toml")
    paths = sorted(LOCALES.glob("*.toml"))
    if not all([check_catalog(path, source) for path in paths]):
        return 1
    print(f"{len(paths)} catálogos válidos.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
