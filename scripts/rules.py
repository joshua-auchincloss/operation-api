from pathlib import Path
from typing import List, Dict, Any, Iterable
import json


ROOT = Path(__file__).resolve().parent.parent
RULES_PATH = ROOT / "rules.json"

TARGET = ROOT / "docs/src/linting/rules.md"


def load_rules() -> List[Dict[str, Any]]:
    return json.loads(RULES_PATH.read_text())


def group_rules(rules: Iterable[Dict[str, Any]]) -> Dict[str, List[Dict[str, Any]]]:
    groups: Dict[str, List[Dict[str, Any]]] = {}
    for r in rules:
        grp = r["group"]
        groups.setdefault(grp, []).append(r)
    return {k: sorted(v, key=lambda x: x["name"]) for k, v in sorted(groups.items())}


def rules_to_markdown(rules: Iterable[Dict[str, Any]]) -> str:
    grouped = group_rules(list(rules))
    lines: List[str] = [
        "# Linting Rules",
        """Rules are given a RuleGroup, default level, and fix level."""
    ]
    for group, items in grouped.items():
        lines.append(f"## group `{group}`")
        lines.append("")
        for rule in items:
            name = rule["name"]
            desc = rule["description"]
            level = rule["level"]
            fix = rule["fix"]

            lines.append(f"### rule `{name}`")
            if desc:
                lines.append(desc)

            lines.append(f"- Default level: {level}")
            lines.append(f"- Fix: {fix}")
            lines.append("")

        lines.append("")

    md = "\n".join(lines).strip() + "\n"
    return md


def generate_markdown() -> str:
    return rules_to_markdown(load_rules())


if __name__ == "__main__":
    TARGET.write_text(
        generate_markdown()
    )
