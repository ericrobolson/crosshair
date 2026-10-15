# SPIKE: Crosshair

Goal: a ratatui editor that can run in a native GUI window (so Cmd keys work), with Cursor-style ghost-text completion driven by warm LLM CLIs. The spike is organised as milestones, ordered by how much each de-risks the project. Everything else is in `FEATURES.md`.

## Decisions already made
- Rust, ratatui, tokio. Repo: `~/dev/crosshair`.
- Frontend is behind ratatui's `Backend` trait: a GUI window (wgpu/egui-based) as the primary target, and crossterm kept as a second target for ssh/tmux.
- Modeless, VS Code-style editing. The input layer is abstracted from actions so a vim keymap can be added later.
- Completion backend is a `Completer` trait. First impl drives a CLI. A direct-API impl must be possible without touching the editor.
- **Pool of 2 warm CLI processes.**
- **Fire only when the lookahead changes**, not on every keystroke or pause.
- Stale output is discarded, not cancelled. A suggestion stays visible while typed characters match its start.
- **Recycle each worker after about 40k tokens** of accumulated context (kill and respawn).
- Strip agent overhead: minimal system prompt, tools disabled, no settings/MCP/CLAUDE.md loading. Exact flags to be confirmed against current CLIs.
- Context store is SQLite, queried by the editor to build the prompt (Milestone 5). Agents do not read it directly.

## Side task: Latency probe (runs alongside Milestone 1)
Standalone binary or script, no UI. It answers whether warm-CLI completion is viable at all, so do it early. Using Haiku, ~20 runs each with a realistic 100-line prefix and 30-line suffix:
1. Spawn per request (`claude -p`), default flags.
2. Spawn per request, stripped flags.
3. Warm `--input-format stream-json --output-format stream-json` process, stripped flags: first turn vs turn N.
4. Same as 3 with context growing toward 40k, to see latency and cost drift.

Record time to first token, time to complete, tokens in/out, and whether the output is clean (insertion only, no commentary). Write findings to `SPIKE-RESULTS.md`.

Go / no-go: if warm p50 is above about 3 s, fall back to explicit-trigger completion, or prioritise the direct-API backend.

---

## Milestone 1: GUI + ratatui proof
Prove that a ratatui app can render in a native window and receive Cmd-key input. This is the most novel and riskiest piece of the UI.

- [x] Select `eframe` 0.34 for the native GUI event loop and rendering.
- [x] Open a native window that renders a ratatui frame (text, colors, borders, dim style).
- [x] Bundle Cascadia Mono at 13 pt and render text through egui's native text pipeline for crisp HiDPI output.
- [x] Keyboard events mapped to a backend-neutral input type, including the Super/Cmd modifier.
- [ ] Demo handles Cmd+F, Cmd+S and Cmd+P and displays normalized key events on screen. The mapping is implemented; manual GUI key delivery still needs confirmation.
- [x] Resize handling with the Ratatui cell grid frozen until resize settles.
- [x] Mouse click and scroll events delivered.
- [x] Same demo app also runs on the crossterm backend, selected by `--backend terminal`.
- [ ] Record frame time and idle CPU use. During resize, the Ratatui layout is frozen until 24 ms after the last size change, then it updates once. Idle CPU still needs a stable sample.

Implementation: `cargo run` launches the eframe GUI by default; `cargo run -- --backend terminal` runs the same app with Crossterm. The GUI renders the Ratatui buffer from `TestBackend` as styled egui text, avoiding a software-rasterized terminal image. See `README.md` for details.

**Done when:** one small app runs in both a GUI window and a terminal, and Cmd+F is received in the GUI.

## Milestone 2: Load a file, file browser
- [ ] `ropey` buffer loaded from a path given on the command line.
- [ ] Read-only viewer: scrolling, line numbers, status bar.
- [ ] Syntax highlighting via tree-sitter (one or two languages).
- [ ] File browser sidebar: tree, expand/collapse, `.gitignore` aware.
- [ ] Open a file from the browser into the viewer.
- [ ] Focus switching between browser and viewer.
- [ ] Large-file sanity check (e.g. 50k lines stays smooth).

**Done when:** you can launch in a directory, browse, and open any text file.

## Milestone 3: Editing
- [ ] Cursor movement, selection, word and line motions.
- [ ] Insert and delete, newline, auto-indent.
- [ ] Undo/redo.
- [ ] Clipboard copy/cut/paste.
- [ ] Save, plus unsaved-changes indicator.
- [ ] Reload on external file change.
- [ ] Mouse click to place cursor, drag to select.
- [ ] Input layer routes keys to named actions (keymap is data, not hard-coded).
- [ ] Fuzzy file finder (Cmd+P) and repo grep (Cmd+Shift+F), if time allows. Otherwise moved to the start of FEATURES.

**Done when:** you can comfortably edit and save real source files in the app.

## Milestone 4: Agent completion
Uses what the latency probe found. If the probe says no-go, adjust scope first.

### 4a. Completer abstraction and ghost text
```rust
trait Completer {
    async fn complete(&self, req: CompletionRequest) -> Result<Completion>;
}
struct CompletionRequest { prefix: String, suffix: String, file_path: PathBuf, extra_context: Vec<ContextChunk> }
struct Completion { text: String, generated_for_prefix_len: usize, usage: Usage }
```
- [ ] `CliCompleter` for one warm process.
- [ ] Ghost text rendered as dim text after the cursor.
- [ ] Tab accepts, Esc dismisses.
- [ ] `ApiCompleter` stub to prove the trait is backend-neutral.
- [ ] Fake `Completer` for deterministic tests.

### 4b. Pool and speculative matching
- [ ] Two warm workers. Dispatch to an idle worker. If both are busy, drop the request (do not queue stale work).
- [ ] Lookahead key = hash of (prefix window, suffix window). Fire only on key change after a short debounce.
- [ ] On each keystroke, if the char matches the head of the current suggestion, trim and keep it. Otherwise discard.
- [ ] Discard results whose generating prefix no longer matches the buffer.
- [ ] Track token usage per worker, recycle at about 40k, and respawn in the background so a warm one is always available.
- [ ] Status bar shows in-flight state, latency and worker health.

Completions in this milestone use only the prefix and suffix around the cursor (`extra_context` left empty). Context building is Milestone 5.

**Done when:** typing in a real file produces ghost-text suggestions that accept with Tab, with measured latency and no runaway token use.

## Milestone 5: Context store (SQLite)
Feeds richer context into completion prompts. Builds on the `extra_context` field from Milestone 4.

- [ ] SQLite schema and migrations.
- [ ] Content hashing to detect changed files.
- [ ] Tree-sitter symbol maps per file (signatures, types, imports).
- [ ] Recent-edits table.
- [ ] Editor queries tables and puts results into the prompt, stable content first and volatile content last.
- [ ] Token budget per context section.
- [ ] Measure: do completions improve, and what does the extra context cost in latency?
- [ ] LLM summaries per directory/file (archival-inspired) in a separate worker pool. Stretch goal; otherwise see `FEATURES.md` section 5.

**Done when:** completions demonstrably use project context (e.g. correct signatures from imported files) without a noticeable latency regression.

## Prompt shape (initial)
System: "You are a code completion engine. Output only the text to insert at <CURSOR>. No explanations, no markdown fences."
User: context chunks, then `<prefix>...<CURSOR>...<suffix>`.

## Open questions to answer during the spike
- Which GUI-backend crate (or custom backend) is solid enough? (Milestone 1)
- Which CLI flags give a minimal-context turn in the current Claude Code and Codex CLIs?
- Does stream-json report token usage per turn?
- Is Haiku through the CLI clean enough at insertion-only output, or do we need post-processing?
- Multi-line vs single-line suggestions: which does the latency allow?
- How much does a 2-process pool help p95?

## Out of scope for the spike
File-tree operations (create/rename/delete), vim mode, LSP, split panes, the long-running agent chat panel. See `FEATURES.md`.
