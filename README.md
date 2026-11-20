# RustroMacro

A lightweight, standalone **Bee Swarm Simulator** macro written in **Rust** — inspired by [NatroMacro](https://github.com/NatroTeam/NatroMacro), built to compile to a single native Windows `.exe` with zero runtime dependencies.

## Features (v0.1)

- **Native Win32 Input (`SendInput`)**: Hardware scancode keyboard & mouse input so Roblox reliably registers movement, camera turns, and tool clicks.
- **Win32 GDI Screen Capture**: Captures the Roblox client window (`BitBlt`) to detect backpack fill %, hive prompt, and window state.
- **Global Hotkeys**: Start/Pause (`F1`) and Stop (`F2`) work even while Roblox is focused.
- **Modular Paths & Patterns**:
  - Starter fields: Pine Tree, Pumpkin, Pepper, Sunflower, Dandelion, Clover, Blue Flower, Strawberry
  - Gathering patterns: `Squares`, `Snake`, `CornerXSnake`, `Stationary`
- **Single-file UI**: Lightweight `egui` (`eframe`) interface with persistent `rustro_config.toml`.

## Build & Run

```powershell
cargo run --release
```

The compiled binary will be at `target/release/rustro_macro.exe`.
