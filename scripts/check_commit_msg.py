"""commit-msg hook: check a commit message against the Conventional Commits and OpenSpec change rules.

Self-contained (stdlib only), so it runs with plain python3 whatever the project stack is.

Usage: python3 scripts/check_commit_msg.py PATH_TO_COMMIT_MSG
"""

import re
import sys
from pathlib import Path

TYPES = ("feat", "fix", "perf", "refactor", "test", "docs", "style", "chore", "build", "ci")

# Commit types that change behaviour and therefore need an OpenSpec change link.
GATED_TYPES = frozenset({"feat", "fix", "perf", "refactor", "test"})

_HEADER = re.compile(
    rf"^(?P<type>{'|'.join(TYPES)})"
    r"(\((?P<scope>[^()\r\n]+)\))?"
    r"(?P<breaking>!)?"
    r": (?P<desc>\S.*)$"
)
_BREAKING_FOOTER = re.compile(r"^BREAKING[ -]CHANGE: \S", re.MULTILINE)
_TRAILER = re.compile(r"^(?P<key>[A-Za-z][A-Za-z0-9-]*): (?P<value>.*)$")
_GIT_GENERATED = re.compile(r"^(Revert \"|Merge (branch|remote-tracking branch|tag|pull request) )")
_SCISSORS = "# ------------------------ >8"


def clean_message(message: str) -> str:
    """Drop what git adds for the editor: everything after the scissors line, then '#' comment lines."""
    lines = message.split("\n")
    for index, line in enumerate(lines):
        if line.startswith(_SCISSORS):
            lines = lines[:index]
            break
    return "\n".join(line for line in lines if not line.startswith("#"))


def trailers(message: str) -> dict[str, list[str]]:
    """Trailers from the final block of the message, grouped by key in order of appearance."""
    blocks = message.rstrip().split("\n\n")
    result: dict[str, list[str]] = {}
    for line in blocks[-1].splitlines():
        match = _TRAILER.match(line)
        if match is not None:
            result.setdefault(match.group("key"), []).append(match.group("value").strip())
    return result


def active_changes(repo: Path) -> set[str]:
    changes = repo / "openspec" / "changes"
    if not changes.is_dir():
        return set()
    return {p.name for p in changes.iterdir() if p.is_dir() and p.name != "archive"}


def check_message(message: str, active: set[str]) -> list[str]:
    """Rule errors for one commit message; an empty list means the message passes."""
    message = clean_message(message)
    first = message.split("\n", 1)[0].rstrip("\r")
    if _GIT_GENERATED.match(first):
        return []
    header = _HEADER.match(first)
    if header is None:
        allowed = ", ".join(TYPES)
        return [
            f"not Conventional Commits: expected '<type>(<scope>)!: <description>' with type one of {allowed}"
        ]
    gated = (
        header.group("type") in GATED_TYPES
        or header.group("breaking") is not None
        or _BREAKING_FOOTER.search(message) is not None
    )
    if not gated:
        return []
    links = trailers(message).get("OpenSpec-Change", [])
    if not links:
        return ["behavioural commit needs an 'OpenSpec-Change: <id>' trailer"]
    return [f"OpenSpec-Change '{link}' is not an active change" for link in links if link not in active]


def main(argv: list[str]) -> int:
    errors = check_message(Path(argv[0]).read_text(), active_changes(Path.cwd()))
    for error in errors:
        print(f"commit-msg: {error}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
