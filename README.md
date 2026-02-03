# RUNE

**RUNE** is a next-generation Vim-style editor designed for power users who demand performance, security, and extreme hackability. Built entirely in Rust, RUNE preserves the classic modal editing philosophy while incorporating a modern UX and a secure, Lua-powered expansion system.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Build Status](https://img.shields.io/badge/status-MVP-orange.svg)
![Language](https://img.shields.io/badge/language-Rust-red.svg)

## 🚀 Vision

RUNE aims to be the bridge between the traditional terminal-based productivity and modern GUI convenience. It features a high-performance rope-based text engine, ensuring ultra-low latency (<10ms) even with large telemetry or log files.

## ✨ Features

- **Modal Editing**: True-to-vim implementation of Normal, Insert, and Command modes.
- **Rope Engine**: High-performance text manipulation powered by `ropey`.
- **Operator-Motion Design**: Fully compatible with `d{motion}`, `y{motion}`, etc.
- **Secure Extensibility**: Sandboxed Lua scripting environment for custom plugins.
- **Modern UI**: Vibrant, color-coded status bar and basic syntax highlighting.
- **Deterministic Config**: Centralized TOML-based configuration for consistent environments.
- **Undo/Redo**: Deep history stack for safe experimentation.

## 🛠 Installation

To build RUNE from source, you'll need the latest stable Rust toolchain.

```bash
git clone https://github.com/ismailtsdln/rune.git
cd rune
cargo build --release
```

The binary will be available at `./target/release/rune`.

## 📖 Quick Start

```bash
./target/release/rune [filename]
```

### Core Shortcuts

| Key | Mode | Description |
| :--- | :--- | :--- |
| `i` | Normal | Enter Insert Mode |
| `Esc` | Insert | Return to Normal Mode |
| `hjkl` | Normal | Move Cursor |
| `w/b` | Normal | Jump by Words |
| `d{m}` | Normal | Delete by motion (e.g., `dw`) |
| `u` | Normal | Undo |
| `Ctrl-r` | Normal | Redo |
| `:` | Normal | Enter Command Mode |
| `/` | Normal | Search Forward |

### Commands

- `:w` - Save File
- `:q` - Quit
- `:wq` - Save and Quit
- `:e <path>` - Edit New File

## 🧪 Development Status

RUNE is currently in the **MVP** phase. Upcoming features include:

- Visual Mode (Range Selection)
- Tree-sitter for semantic highlighting
- WASM-based plugin architecture
- Multiple window splits and tabs

## 📜 License

This project is licensed under the [MIT License](LICENSE).
