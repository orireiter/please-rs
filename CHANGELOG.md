# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Project Snapshot (2026-03-13)

This changelog is intentionally developer-facing. It is meant to help resume work quickly.

### Current Health

- `cargo build` passes on Windows (dev profile).
- Build currently has warnings (unused vars/variants and unfinished tab context plumbing), but no errors.
- Interactive shell loop is usable for basic command execution and line editing.

### What Works

- Terminal raw-mode shell loop with custom prompt (`<cwd> -> `).
- History persistence to `~/.please_history`, plus up/down navigation with prefix filtering.
- Cursor-aware editing in the command line:
	- left/right navigation.
	- ctrl+left/right word jumps.
	- backspace and ctrl+backspace behavior.
	- ctrl+c clears current input and returns to prompt.
- Command execution pipeline:
	- internal `please exit` command.
	- native commands: `clear`, `ls`, and `cd`.
	- fallback process spawning with PATH lookup.
	- quote-aware argument grouping for external commands.
- Initial tab completion scaffolding and completion candidate model.

### In Progress / Partially Wired

- Multi-candidate tab UI (`TabContext`) can render and move selection with left/right.
- `handle_tab` creates/runs tab context but currently does not apply the selected completion back into the live command.
- Directory completion provider mostly returns entries from current directory; argument/path-specific completion is still marked TODO.

### Known Gaps

- No support yet for operators/pipelines like `&&` and `|`.
- Paste and mouse terminal events are still TODO.
- Completion concat strategy (`PrefixConcat`) is defined but unused.
- History filtering/navigation exists, but broader UX refinements remain.
- Installation/packaging workflow is not implemented yet.

### Local Working Tree Notes

- Current unstaged edits exist in:
	- `src/commands/completion.rs`
	- `src/commands/traits.rs`
	- `src/terminal/tab_context.rs`
- This file (`CHANGELOG.md`) is new.

### Suggested Next Steps (Resume Order)

1. Finish tab completion end-to-end: apply `TabResult::AppendText` into the live command and redraw correctly.
2. Implement path-aware completion (last argument expansion, partial overlap suffix logic).
3. Reduce warnings by removing/using currently dead variants and fields as behavior is finalized.
4. Add operator parsing (`&&`, `|`) in command execution.
5. Add minimal regression checks for history navigation and quote parsing.

## [History]

### 2026-03

- Added tab context runner for completion navigation.
- Added left-movement utility to support cursor manipulation.

### 2025-10

- Created command sub-module split.
- Added tab completion traits and initial directory completion provider.

### 2025-08

- Added `handle_tab` to key handling flow.
- Added ctrl+c handling.
- Implemented `ls` native command.
- Fixed multiline command writing and interactive command input behavior.
- Fixed history end-boundary behavior.
- Improved quote handling for command execution.
- Added ctrl+arrow quick jumps.

### 2025-07

- Initial project foundation and terminal organization.
- Added env_logger.
- Added native command support and clear command wiring.
- Implemented history and `chdir` support.
- Added initial README and roadmap TODOs.
