# Changelog

All notable changes to this project will be documented in this file.

## [0.1.4] - 2026-04-18

### Fixed

- REPL syntax highlighting: arrows after node names now highlight correctly

## [0.1.3] - 2026-04-18

### Added

- REPL syntax highlighting for commands, keywords, arrows, and strings
- Tab completion for REPL commands, themes, and file paths
- Line validation and history hints in REPL

## [0.1.2] - 2026-04-18

### Added

- Full REPL command system with `rustyline` for line editing and persistent history
- `:theme` command to switch diagram themes (default, dark, forest, neutral, base)
- `:save` command to export last rendered HTML to a file
- `:load` command to render diagrams from `.mmd` files
- `:last` command to re-render the previous diagram with current theme
- `:begin` / `:end` for multi-line diagram input
- `:clear` command to remove session temp files
- `:help` command for REPL reference
- Arrow-key history persisted across sessions

## [0.1.1] - 2026-04-18

### Added

- Interactive REPL mode when launched without arguments
- `--help` / `-h` flag with usage documentation
- Automatic stripping of wrapper quotes (`""`, `''`, `` ` ``) in REPL mode
- Safe pipe mode: exits with error on empty input when not a terminal

## [0.1.0] - 2026-04-18

### Added

- Render Mermaid diagrams in the browser from CLI args or stdin
- Pan and zoom support in the browser viewer
- Pre-built binaries for Windows (x64), macOS (Intel + Apple Silicon), Linux (x64 + ARM64)
