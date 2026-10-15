# Crosshair

Crosshair is a Ratatui editor spike. Milestone 1 has a native `eframe` window that paints Ratatui's buffer through egui, plus a terminal mode using Crossterm. Both modes call the same view and share the same backend-neutral input types.

## Run

```sh
cargo run                 # native GUI window
cargo run -- --backend terminal
```

In the GUI, press Cmd+F, Cmd+S, or Cmd+P and check the status line for the normalized key event. Click or scroll in the view to see mouse events. Escape closes the app. The GUI uses bundled Cascadia Mono at 13 pt through egui's text renderer. The scale factor and last draw time appear in the view. `walkdir` is included for the upcoming project file browser.

The eframe window waits for events while idle and draws on input or resize. During a live resize, it holds the Ratatui cell grid and paints it directly with egui, then updates the grid once 24 ms pass without a size change. The GUI uses Ratatui's in-memory `TestBackend`; the terminal mode uses Crossterm. `walkdir` is included for the upcoming project file browser.

