# rvim

A minimal Vim clone in Rust — terminal-based modal text editor.

## Quick start

```bash
cargo run -- --help
cargo run -- file.txt
```

## Features

- **Modal editing**: Normal, Insert, Visual, Search, and Command-line modes
- **Vim key bindings**: `h/j/k/l/w/b/e/0/$/^/gg/G/%` motion,
  `d/y/c` with counts and linewise/range operators, `i/I/a/A/o/O`,
  `x/X/s/S/D/C/J/r/~`, `u/Ctrl+r` undo/redo, `p/P` paste, `.` repeat,
  `v` visual with `d/y/c/>/<` operations, `/?` search with `n/N`
- **Multi-buffer**: open several files, switch with `:bn`/`:bp`/`:ls`
- **Menu bar**: File, Edit, View, Search, Tools — open with `Alt+letter`
  or `Ctrl+F10`
- **Global shortcuts**: `F1` help, `F2` toggle menu, `F3` line numbers,
  `F4` relative numbers, `Ctrl+s/z/y/n/o/q` for common actions
- **Pagination popups**: Help dialog, About dialog with GPL license viewer,
  buffer list with number-jump
- **Undo/redo**: nested batch support for correct insert grouping
- **Clipboard**: unnamed register, linewise paste
- **Config file**: `~/.config/rvim/config.toml` with commented options
- **Search**: forward/backward, wraps around file, `hlsearch`/`ignorecase`/
  `smartcase` support
- **File dialogs**: Save, Save As, Open with centered box UI
- **`:set` options**: `number`, `relativenumber`, `hlsearch`, `ignorecase`,
  `smartcase`, `menu`, `tabstop`, `shiftwidth`, `scrolloff`

## Build

```bash
cargo build
cargo run -- <files...>
```

Requires a terminal with crossterm support (Linux, macOS, Windows).

## Configuration

Options are read from `~/.config/rvim/config.toml`. The file is created
automatically on first launch with all options documented.

## License

GNU General Public License v3 or later. See `license.txt`.
