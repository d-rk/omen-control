# omen-control — working rules

A desktop UI for the [omen-rgb-keyboard](https://github.com/OmenLinux/omen-rgb-keyboard) driver.

## Where things live

- Specs: `openspec/` (OpenSpec, root location). Each change is `openspec/changes/<id>/`.
- Application code, tests and check commands: decided with the first change that picks the stack. Update this file
  when they exist.
- Repository tooling: `scripts/`, git hooks in `.githooks/`.

## Workflow for every behavioural change

1. Pick an active change under `openspec/changes/`, or propose one first (`/opsx:propose`). Do not change
   application code or tests without an active change.
2. Write a failing test and commit it on its own. This workflow is test-first. It is a convention, not checked
   mechanically.
3. Write the minimal implementation and commit it.
4. Make sure lint, format check and the fast test suite pass before committing.

## Commit messages

Conventional Commits 1.0.0, with one of these types: `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `style`,
`chore`, `build`, `ci`.

- `feat`, `fix`, `perf`, `refactor`, `test` and any breaking commit need an `OpenSpec-Change: <id>` trailer naming an
  active change.
- `docs`, `style`, `chore`, `build` and `ci` need no trailer. Pick the type that matches what the change does,
  because the type decides which gates apply.

The `commit-msg` hook (`scripts/check_commit_msg.py`) enforces both rules; do not use `--no-verify`.

## Test policy

- Write tests only for behaviour that a scenario in the spec describes, or for the decision logic such a scenario
  depends on. Do not write tests that check wording, formatting, or contents of prose or config files.
- Every test names the scenario it covers, using the exact name from the spec. A test that names no scenario is
  removed. If the behaviour matters but no scenario covers it, add the scenario to the spec first, then the test.
- TDD applies to decisions: branching logic and interfaces other code depends on. Config, glue and docs are checked
  by running them once, not by a test.
- Test each rule at the lowest layer that owns it. Test the wiring between layers once.
- Before adding a test, check whether an existing test already covers the scenario.

## Setup

Run `git config core.hooksPath .githooks` once per clone.
