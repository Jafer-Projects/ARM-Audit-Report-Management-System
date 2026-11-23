#!/usr/bin/env python3
"""Dependency-free structural validation for the ARM specification repository."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REQUIRED_FILES = (
    "README.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "docs/product/software-requirements-specification.md",
    "docs/architecture/system-architecture.md",
    "docs/architecture/offline-first-and-sync.md",
    "docs/governance/architecture-decision-records.md",
    "docs/discovery/phase-0-technical-discovery-plan.md",
    "docs/discovery/open-questions-and-evidence-register.md",
    "docs/discovery/technical-spike-and-validation-matrix.md",
    "docs/governance/phase-0-gate-record.md",
)
LINK_RE = re.compile(r"!?(?:\[[^\]]*\])\(([^)]+)\)")


def main() -> int:
    errors: list[str] = []
    for relative in REQUIRED_FILES:
        if not (ROOT / relative).is_file():
            errors.append(f"Missing required file: {relative}")

    fence = chr(96) * 3
    for markdown in sorted(ROOT.rglob("*.md")):
        in_fence = False
        for number, line in enumerate(markdown.read_text(encoding="utf-8").splitlines(), 1):
            if line.lstrip().startswith((fence, "~~~")):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            for destination in LINK_RE.findall(line):
                destination = destination.strip().split(maxsplit=1)[0].strip("<>")
                if not destination or destination.startswith(("#", "http://", "https://", "mailto:", "data:")):
                    continue
                path_part = destination.split("#", 1)[0].split("?", 1)[0]
                if not path_part:
                    continue
                target = (markdown.parent / path_part).resolve()
                try:
                    target.relative_to(ROOT.resolve())
                except ValueError:
                    errors.append(f"{markdown.relative_to(ROOT)}:{number}: link escapes repository: {destination}")
                    continue
                if not target.exists():
                    errors.append(f"{markdown.relative_to(ROOT)}:{number}: broken local link: {destination}")

    if errors:
        print("Repository validation FAILED:")
        for error in errors:
            print(f"- {error}")
        return 1
    print(f"Repository validation PASSED: {len(REQUIRED_FILES)} required files and local Markdown links checked.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
