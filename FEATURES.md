# Crosshair Features and Roadmap

A terminal-style text editor (ratatui + tokio) with Cursor-like tab completion driven by LLM CLIs or APIs, and a local context store.

Legend: `[ ]` todo, `[x]` done. Items marked (spike) are covered in `SPIKE.md`.

## 0. Spike (see SPIKE.md)
- [ ] Latency probe: spawn vs warm CLI, Haiku
- [ ] Minimal ratatui editor with ghost text
- [ ] `Completer` trait with CLI impl (pool of 2)
- [ ] Fire only on lookahead change
- [ ] Speculative prefix matching (typed-along keeps suggestion)
- [ ] Stale-output discard
- [ ] Recycle workers at about 40k tokens
- [ ] Stripped-down CLI flags (no tools, no settings)

## 1. Editor core
- [ ] Rope buffer (`ropey`), multi-buffer
- [ ] Cursor movement, selection, word/line motions
- [ ] Undo/redo tree
- [ ] Clipboard integration
- [ ] Open, save, save-as, unsaved-changes prompt
- [ ] External file change detection and reload (`notify`)
- [ ] Soft wrap and horizontal scroll
- [ ] Line numbers, status bar
- [ ] Tabs / split panes
- [ ] Syntax highlighting (tree-sitter)
- [ ] Auto-indent, bracket matching, auto-close pairs
- [ ] Unicode and wide-char correctness
- [ ] Config file (keymap, theme, backend)

## 2. Navigation and search
- [ ] File browser / tree sidebar
- [ ] Fuzzy file-name finder (`nucleo`)
- [ ] Repo-wide text grep (`ignore` + `grep` crates), results list, jump to line
- [ ] In-file find and replace
- [ ] Replace across files with preview
- [ ] Respect `.gitignore`
- [ ] Recent files list
- [ ] Go to line, go to symbol (tree-sitter)

## 3. Frontend and input
- [ ] `Backend` abstraction over crossterm and a GUI window
- [ ] GUI backend prototype (wgpu/egui ratatui crate, or custom winit + cosmic-text)
- [ ] Cmd+key bindings in GUI (Cmd+F, Cmd+S, Cmd+P, Cmd+Shift+F)
- [ ] Font config, ligatures, HiDPI
- [ ] Keep crossterm target working for ssh/tmux
- [ ] Input layer abstracted from actions (for vim mode later)
- [ ] Modeless VS Code-style keymap (v1)
- [ ] Vim modal keymap (later)
- [ ] Mouse support: click, drag-select, scroll

## 4. Completion
- [ ] Multi-line suggestions
- [ ] Partial accept (word / line)
- [ ] Per-language prompt tweaks
- [ ] Direct API backend (`ApiCompleter`)
- [ ] Codex CLI backend
- [ ] Local model backend (Ollama or similar)
- [ ] Rate limiting and quota guard
- [ ] Per-backend latency and token metrics, shown in status bar
- [ ] Suggestion cache keyed on lookahead
- [ ] Next-edit prediction (jump to the next likely edit)
- [ ] Toggle completion per file type or project

## 5. Context store (SQLite)
Core pieces (schema, hashing, symbol maps, recent edits, prompt assembly) are Milestone 5 in `SPIKE.md`. The rest is here.
- [ ] Schema and migrations
- [ ] Tree-sitter symbol maps per file (signatures, types, imports)
- [ ] Content hashing to detect changes
- [ ] LLM summaries per directory and file, regenerated on hash change (inspired by archival)
- [ ] Separate worker pool for summarization (never blocks completions)
- [ ] Edit history / recent changes table
- [ ] Prompt assembly from tables: stable context first, volatile last
- [ ] Token budgeting per context section
- [ ] Indexing status in status bar
- [ ] Pause indexing while typing
- [ ] `--max-dirs`-style incremental bootstrap for large repos
- [ ] Ignore rules for the store
- [ ] Optional markdown export for agents that read files
- [ ] Optional `crosshair query` CLI or MCP server for external agents
- [ ] Embeddings or FTS for semantic lookup (later)

## 6. Agent integration (later)
- [ ] Agent panel driving a long-running Claude Code / Codex CLI (stream-json)
- [ ] Send selection, buffers and diagnostics as context
- [ ] Diff view with accept/reject for agent edits
- [ ] Git checkpoint before each agent turn
- [ ] Handling of concurrent user edits while the agent writes
- [ ] ACP client support

## 7. Language tooling
- [ ] LSP client (diagnostics, hover, go to definition)
- [ ] Diagnostics in gutter
- [ ] Format on save

## 8. Quality and release
- [ ] Unit tests for buffer, matching, pool logic
- [ ] Fake `Completer` for deterministic tests
- [ ] Benchmarks for large files
- [ ] CI (fmt, clippy, test)
- [ ] README with install steps
- [ ] Logging and crash reports
- [ ] Packaging (`cargo install`, Homebrew)
