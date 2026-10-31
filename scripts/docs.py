#!/usr/bin/env python3
"""Generate the public documentation indexes, or check their exact freshness."""
from pathlib import Path
import argparse

ROOT = Path(__file__).resolve().parents[1]
DOCS = [
    ("README.md", "Overview and first render"),
    ("docs/getting-started.md", "Installation and native/browser use"),
    ("docs/architecture.md", "Shared generation and platform ownership"),
    ("docs/api.md", "Rust, Python, C and WebAssembly interfaces"),
    ("docs/frame-contract.md", "Bounded descriptors, RGBA pixels and lifecycle"),
    ("docs/catalog.md", "Style identifiers, activity and provenance"),
    ("docs/faq.md", "Format and integration questions"),
    ("docs/troubleshooting.md", "Setup, input and resource errors"),
]
CONTRIBUTING = [
    ("CONTRIBUTING.md", "Development and compatibility"),
    ("SECURITY.md", "Vulnerability reporting and trust boundaries"),
    ("CODE_OF_CONDUCT.md", "Contributor expectations"),
    ("LICENSE", "MIT license"),
]
OPTIONAL = [
    ("CHANGELOG.md", "User-visible changes"),
    ("ACKNOWLEDGEMENTS.md", "Credits and design lineage"),
]


def outputs():
    index = ["# AbstractAvatar", "", "> Shared procedural avatars for native, terminal and browser applications.", ""]
    for title, entries in [("Core documentation", DOCS), ("Contributing", CONTRIBUTING)]:
        index.extend([f"## {title}", ""])
        index.extend(f"- [{path}]({path}): {summary}" for path, summary in entries)
        index.append("")
    index.extend(["## Agent skill", "", "- [AbstractAvatar](skills/abstractavatar/SKILL.md): safe rendering and integration workflow", "", "## Optional", ""])
    index.extend(f"- [{path}]({path}): {summary}" for path, summary in OPTIONAL)
    corpus = DOCS + CONTRIBUTING + OPTIONAL
    full = ["# AbstractAvatar", "", "> Shared procedural avatars for native, terminal and browser applications.", "", "## Document index", ""]
    full.extend(f"- [{path}]({path}): {summary}" for path, summary in corpus)
    for path, _ in corpus:
        full.extend(["", "---", "", f"## {path}", "", (ROOT / path).read_text(encoding="utf-8").rstrip()])
    return {"llms.txt": "\n".join(index).rstrip() + "\n", "llms-full.txt": "\n".join(full).rstrip() + "\n"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Fail when generated indexes are missing or stale")
    args = parser.parse_args()
    stale = []
    for name, text in outputs().items():
        path = ROOT / name
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != text:
                stale.append(name)
        else:
            path.write_text(text, encoding="utf-8")
    if stale:
        parser.exit(1, "Stale documentation indexes: " + ", ".join(stale) + "\n")
    print("Documentation indexes are current" if args.check else "Documentation indexes generated")


if __name__ == "__main__":
    main()
