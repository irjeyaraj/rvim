# rvim User Guide

rvim is a minimal Vim clone written in Rust. It runs in the terminal and
supports modal editing, multi-buffer management, a menu bar, configurable
options, and common Vim key bindings.

---

## Table of Contents

1. [Getting Started](#1-getting-started)
2. [Modes Overview](#2-modes-overview)
3. [Normal Mode](#3-normal-mode)
4. [Insert Mode](#4-insert-mode)
5. [Visual Mode](#5-visual-mode)
6. [Search Mode](#6-search-mode)
7. [Command-Line Mode](#7-command-line-mode)
8. [Command Reference](#8-command-reference)
9. [Configuration](#9-configuration)
10. [Menu Bar](#10-menu-bar)
11. [Dialogs](#11-dialogs)
12. [Global Shortcuts](#12-global-shortcuts)
13. [Buffer Management](#13-buffer-management)
14. [Appendices](#14-appendices)

---

## 1. Getting Started

### 1.1 Running rvim

```bash
rvim                     # start with an empty buffer
rvim file1.txt           # open one file
rvim file1.txt file2.txt # open multiple files (buffers)
```

### 1.2 Configuration file

rvim reads options from `~/.config/rvim/config.toml`. This file is created
automatically on first launch with all options documented and set to their
defaults. Edit it with any text editor and restart rvim to apply changes.

### 1.3 Exiting

- `:q` — quit (rejects if unsaved changes)
- `:q!` — force quit
- `:wq` — save and quit
- `Ctrl+Q` — quick quit (same as `:q`)

---

## 2. Modes Overview

rvim is a **modal editor**. The mode is shown in the status bar
(second-to-last line):

| Mode | Status bar label | Purpose |
|---|---|---|
| Normal | `NORMAL` | Navigation, operations, commands |
| Insert | `INSERT` | Typing text |
| Visual | `VISUAL` | Selecting text |
| Search | `SEARCH` | Entering search pattern |
| Command-line | (none — at bottom row) | Entering `:` commands |

Press `Escape` from any mode to return to Normal.

---

## 3. Normal Mode

This is the default mode. Keys perform navigation and text operations.

### 3.1 Numeric Prefix

Typing `1`–`9` before a command sets a repeat count. For example:

```
3j        Move down 3 lines
5w        Move forward 5 words
d3j       Delete 3 lines (current + 2 below)
```

`0` when no count has been typed moves to line start.

### 3.2 Cursor Motion

| Key | Action |
|---|---|
| `h` | Left (wraps to previous line end) |
| `j` | Down |
| `k` | Up |
| `l` | Right (wraps to next line start) |
| `w` | Forward one word |
| `b` | Back one word |
| `e` | End of word |
| `ge` | End of word backwards |
| `0` | Line start (column 0) |
| `$` | Line end |
| `^` | First non-blank character |
| `G` | Go to file end |
| `gg` | Go to file start |
| `%` | Matching bracket `()`, `[]`, `{}` |
| `Ctrl+f` | Page down (one screenful) |
| `Ctrl+b` | Page up (one screenful) |
| `Ctrl+d` | Half page down |
| `Ctrl+u` | Half page up |

**Word definition**: words are sequences of alphanumeric + underscore,
non-word non-whitespace characters (punctuation), or whitespace runs.

### 3.3 Delete, Yank, Change

These three operations follow the pattern `{operator}{motion}`. They accept
a numeric prefix and can be doubled for linewise operation.

| Key | Action |
|---|---|
| `d{motion}` | Delete text covered by motion |
| `dd` | Delete line (accepts count) |
| `D` | Delete to end of line |
| `y{motion}` | Yank (copy) text |
| `yy` | Yank line (accepts count) |
| `c{motion}` | Change (delete + enter Insert) |
| `cc` | Change line (accepts count) |
| `C` | Change to end of line |

Linewise variants also trigger when the motion is `j` or `k`
(e.g. `dj` deletes current and line below).

### 3.4 Text Change Commands

| Key | Action |
|---|---|
| `x` | Delete character at cursor |
| `X` | Delete character before cursor (backspace) |
| `s` | Substitute character (delete + Insert) |
| `S` | Substitute line (change line + Insert) |
| `J` | Join next line onto current |
| `r{c}` | Replace character at cursor with `{c}` |
| `~` | Toggle case at cursor, move right |
| `u` | Undo |
| `Ctrl+r` | Redo |
| `.` | Repeat last change |
| `p` | Paste after cursor (below if linewise) |
| `P` | Paste before cursor (above if linewise) |

### 3.5 Entering Insert Mode

| Key | Action |
|---|---|
| `i` | Insert before cursor |
| `I` | Insert at first non-blank of line |
| `a` | Append after cursor |
| `A` | Append at end of line |
| `o` | Open line below and insert |
| `O` | Open line above and insert |

### 3.6 Searching

| Key | Action |
|---|---|
| `/` | Start forward search |
| `?` | Start backward search |
| `n` | Repeat last search forward |
| `N` | Repeat last search backward |

See [Search Mode](#6-search-mode) for search behaviour details.

### 3.7 Entering Visual Mode

| Key | Action |
|---|---|
| `v` | Start character-wise Visual selection |

### 3.8 Normal-Mode Ctrl Shortcuts

| Key | Action |
|---|---|
| `Ctrl+s` | Save |
| `Ctrl+q` | Quit |
| `Ctrl+n` | New buffer |
| `Ctrl+o` | Open file (file dialog) |
| `Ctrl+z` | Undo |
| `Ctrl+y` | Redo |

---

## 4. Insert Mode

| Key | Action |
|---|---|
| `Escape` | Return to Normal (cursor moves left) |
| `Enter` | Split line |
| `Backspace` | Delete character before cursor (joins lines at BOL) |
| `Delete` | Delete character at cursor (joins next line at EOL) |
| `Tab` | Insert `tabstop` spaces (default 8) |
| `Left` / `Right` / `Up` / `Down` | Arrow navigation |
| `Home` | Go to column 0 |
| `End` | Go to end of line |

All printable characters insert themselves at the cursor. Each character
is a separate undo step.

---

## 5. Visual Mode

Select text character-by-character. Motions extend or shrink the selection.

| Key | Action |
|---|---|
| `Escape` | Return to Normal |
| `h` / `j` / `k` / `l` | Extend selection |
| `w` / `b` / `e` | Extend by word |
| `0` / `$` / `^` | Extend to line bounds |
| `G` | Extend to file end |
| `d` or `x` | Delete selection |
| `y` | Yank selection |
| `c` or `s` | Change selection (delete + Insert) |
| `>` | Indent (add one space at line starts) |
| `<` | Outdent (remove one leading space) |
| `u` | Lowercase selection |
| `U` | Uppercase selection |

---

## 6. Search Mode

Press `/` (forward) or `?` (backward) from Normal mode.

| Key | Action |
|---|---|
| `Escape` / `Ctrl+c` | Cancel search |
| `Enter` | Execute search |
| `Backspace` | Delete last character (cancel if empty) |
| Printable chars | Build search pattern |

Search behaviour:

- **Wraps** around the file (after last line → first, and vice versa).
- **Case sensitivity** is controlled by two config options:
  - `ignorecase`: when `true`, ignore case in patterns
  - `smartcase`: when `true` and pattern contains uppercase,
    `ignorecase` is overridden and search becomes case-sensitive
- All matches are highlighted when `hlsearch` is `true`.

---

## 7. Command-Line Mode

Press `:` from Normal mode. The prompt appears on the last terminal row.

| Key | Action |
|---|---|
| `Escape` / `Ctrl+c` | Cancel (return to Normal) |
| `Enter` | Execute command |
| `Backspace` | Delete character (cancel if empty) |

See [Command Reference](#8-command-reference) for all commands.

---

## 8. Command Reference

### 8.1 Quitting and Saving

| Command | Aliases | Action |
|---|---|---|
| `:q` | `:quit` | Quit (rejects if modified) |
| `:q!` | `:quit!` | Force quit |
| `:w` | `:write` | Save current file |
| `:w!` | | Force write |
| `:wq` | | Save and quit |
| `:wq!` | | Force save and quit |
| `:wa` | `:wall` | Write all buffers |
| `:qa` | `:qall` | Quit all (rejects if modified) |
| `:qa!` | `:qall!` | Force quit all |

### 8.2 Files

| Command | Aliases | Action |
|---|---|---|
| `:e <file>` | `:edit` | Open file in new buffer |
| `:new` | | Create new buffer |
| `:vnew` | `:vne` | Create new buffer |

### 8.3 Buffers

| Command | Aliases | Action |
|---|---|---|
| `:bn` | `:bnext` | Next buffer |
| `:bp` | `:bprev`, `:bprevious` | Previous buffer |
| `:ls` | `:buffers` | List buffers |
| `:only` | | Close all other buffers |

The `:ls` command opens an interactive buffer list. Type a buffer number
and press Enter to jump to it.

### 8.4 Options

| Command | Action |
|---|---|
| `:set` | Show all options in status bar |
| `:set <option>` | Enable boolean option |
| `:set no<option>` | Disable boolean option |
| `:set <option>=<value>` | Set option to value |
| `:nohlsearch` | `:noh` `:nohl` Clear search highlighting |
| `:help` | `:h` Show help |

See the [Configuration](#9-configuration) section for all option names.

---

## 9. Configuration

Options are read from `~/.config/rvim/config.toml` on startup.

### 9.1 Display Options

| Option | Aliases | Default | Toggle off | Description |
|---|---|---|---|---|
| `number` | `nu` | `true` | `nonumber` | Show line numbers |
| `relativenumber` | `rnu` | `false` | `norelativenumber` | Show relative numbers |
| `menu` | | `true` | `nomenu` | Show menu bar |

### 9.2 Search Options

| Option | Aliases | Default | Toggle off | Description |
|---|---|---|---|---|
| `hlsearch` | `hls` | `true` | `nohlsearch` | Highlight all matches |
| `ignorecase` | `ic` | `false` | `noignorecase` | Ignore case |
| `smartcase` | `scs` | `true` | `nosmartcase` | Override ignorecase when pattern has uppercase |

### 9.3 Indentation Options

| Option | Aliases | Default | Description |
|---|---|---|---|
| `tabstop` | `ts` | `8` | Spaces per Tab |
| `shiftwidth` | `sw` | `8` | Spaces for indent step (`<<`, `>>`) |

### 9.4 Scrolling Options

| Option | Aliases | Default | Description |
|---|---|---|---|
| `scrolloff` | `so` | `0` | Min lines above/below cursor |

---

## 10. Menu Bar

The menu bar appears at the top of the terminal. It has five menus:
**File**, **Edit**, **View**, **Search**, **Tools**.

### 10.1 Opening a menu

- **Alt+letter**: `Alt+F` (File), `Alt+E` (Edit), `Alt+V` (View),
  `Alt+S` (Search), `Alt+T` (Tools)
- **Ctrl+F10**: open the File menu
- **Click** with a mouse-aware terminal emulator is not supported.

### 10.2 Navigation

| Key | Action |
|---|---|
| `Left` / `Right` | Switch between menus |
| `Up` / `Down` | Navigate items |
| `Enter` | Activate item |
| `Escape` | Close menu |

### 10.3 File menu

| Item | Shortcut | Action |
|---|---|---|
| New | `Ctrl+N` | New buffer |
| Open… | `Ctrl+O` | File dialog |
| Save | `Ctrl+S` | Save |
| Save As… | | Save with new name |
| *(separator)* | | |
| Close | | Close buffer |
| Quit | `Ctrl+Q` | Quit |

### 10.4 Edit menu

| Item | Shortcut | Action |
|---|---|---|
| Undo | `Ctrl+Z` | Undo |
| Redo | `Ctrl+Y` | Redo |
| *(separator)* | | |
| Copy | | Yank current line |
| Paste | | Paste clipboard |
| *(separator)* | | |
| Delete Line | `dd` | Delete current line |

### 10.5 View menu

| Item | Action |
|---|---|
| Line Numbers | Toggle line numbers |
| Relative Numbers | Toggle relative numbers |
| *(separator)* | |
| Menu Bar | Toggle menu bar visibility |

### 10.6 Search menu

| Item | Shortcut | Action |
|---|---|---|
| Find… | `/` | Forward search |
| Find Next | `n` | Repeat forward |
| Find Prev | `N` | Repeat backward |

### 10.7 Tools menu

| Item | Shortcut | Action |
|---|---|---|
| Help | `F1` | Help dialog |
| *(separator)* | | |
| About rvim | | About dialog |

---

## 11. Dialogs

### 11.1 Help (`F1` or Tools → Help)

A paginated popup listing all key bindings. Navigate with arrows,
PageUp/PageDown, Home/End. Close with `Esc`, `q`, or `Ctrl+c`.

### 11.2 About (Tools → About rvim)

Shows program name, copyright, warranty notice. Press `Ctrl+L` inside the
dialog to view the full GPLv3 license text (paginated). Close with `Esc`,
`q`, or `Ctrl+c`.

### 11.3 File Dialog (Open / Save As)

A centered box with a text input. Type a filename and press Enter.

- If the file exists, it is opened in a new buffer.
- If it does not exist, a new empty buffer is created with that filename.
- Cancel with `Esc`.

### 11.4 Buffer List (`:ls` or `:buffers`)

Lists all open buffers. The current buffer is marked with `>>`. Type a
buffer number and press Enter to switch. Navigate with arrows, PageUp/PageDown,
Home/End. Close with `Esc`, `q`, or `Ctrl+c`.

---

## 12. Global Shortcuts

These shortcuts work in any mode:

| Key | Action |
|---|---|
| `F1` | Help dialog |
| `F2` | Toggle menu bar visibility |
| `F3` | Toggle line numbers |
| `F4` | Toggle relative line numbers |
| `Ctrl+F10` | Activate menu bar (opens File menu) |
| `Alt+{f,e,v,s,t}` | Open corresponding menu |

---

## 13. Buffer Management

rvim supports multiple open files as **buffers**. Each file or new document
occupies one buffer. The current buffer is displayed; only one buffer is
visible at a time.

### 13.1 Creating and opening

- Create a new buffer: `:new`, `Ctrl+N`, or File → New
- Open a file: `:e <file>`, `Ctrl+O`, or File → Open…
- Open files at startup: `rvim file1.txt file2.txt`

### 13.2 Switching buffers

- `:bn` / `:bnext` — next buffer
- `:bp` / `:bprev` / `:bprevious` — previous buffer
- `:ls` / `:buffers` — interactive buffer list

### 13.3 Saving

- `:w` / `Ctrl+S` — save current buffer
- `:wa` / `:wall` — save all buffers
- File → Save As… — save with a new filename

### 13.4 Closing

- File → Close — close current buffer (prevented if it is the last)
- `:only` — close all buffers except the current one

---

## 14. Appendices

### 14.1 Clipboard

rvim uses a single unnamed register. The following operations populate it:

- Delete operations: `x`, `X`, `d`, `D`, `dd`, `c`, `C`, `cc`, `s`, `S`
- Yank operations: `y`, `Y`, `yy`

Paste with `p` (after cursor / below line) and `P` (before cursor / above line).
Multi-line text (containing newlines) is pasted as whole lines.

### 14.2 Undo / Redo

- `u` / `Ctrl+Z` — undo
- `Ctrl+r` / `Ctrl+Y` — redo

Each Insert-mode character is one undo step. Operations like `dd`, `cc`,
paste, and `J` are grouped as single undo steps.

The `.` command repeats the last undo batch.

### 14.3 Search details

- Search wraps around the file.
- `find_all()` is called after each search to collect all match positions
  for highlighting.
- Case sensitivity follows `ignorecase` + `smartcase` logic:
  1. If `smartcase=true` and pattern contains an uppercase letter → case
     sensitive.
  2. Otherwise → follow `ignorecase`.

### 14.4 Bracket matching

The `%` key jumps between matching pairs:

- `(` ↔ `)`
- `[` ↔ `]`
- `{` ↔ `}`

It works in both the forward and backward direction, searching from the
cursor position.

### 14.5 Key types recognised

All key types that rvim can read:

| Key type | Examples |
|---|---|
| `Char(c)` | `a`, `Z`, `3`, `/`, space |
| `Ctrl(c)` | `Ctrl+c`, `Ctrl+v` |
| `Alt(c)` | `Alt+f`, `Alt+x` |
| `Enter` | Return |
| `Tab` | Tab |
| `Backspace` | Delete backwards |
| `Escape` | Esc |
| `Left` / `Right` / `Up` / `Down` | Arrow keys |
| `Home` / `End` | Home, End |
| `PageUp` / `PageDown` | Page Up, Page Down |
| `Delete` | Delete forward |
| `Insert` | Insert key |
| `F(n)` | F1–F12 |
| `CtrlF(n)` | Ctrl+F10 |
