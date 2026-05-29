/*************************************************************************
    rvim — A minimal Vim clone in Rust

    Copyright (C) 2026 Immanuel Jeyaraj <irj@sefier.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*************************************************************************/

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_number")]
    pub number: bool,
    #[serde(default = "default_relativenumber")]
    pub relativenumber: bool,
    #[serde(default = "default_tabstop")]
    pub tabstop: usize,
    #[serde(default = "default_shiftwidth")]
    pub shiftwidth: usize,
    #[serde(default = "default_hlsearch")]
    pub hlsearch: bool,
    #[serde(default = "default_ignorecase")]
    pub ignorecase: bool,
    #[serde(default = "default_smartcase")]
    pub smartcase: bool,
    #[serde(default = "default_scrolloff")]
    pub scrolloff: usize,
    #[serde(default = "default_menu")]
    pub menu: bool,
}

fn default_number() -> bool {
    true
}
fn default_relativenumber() -> bool {
    false
}
fn default_tabstop() -> usize {
    8
}
fn default_shiftwidth() -> usize {
    8
}
fn default_hlsearch() -> bool {
    true
}
fn default_ignorecase() -> bool {
    false
}
fn default_smartcase() -> bool {
    true
}
fn default_scrolloff() -> usize {
    0
}
fn default_menu() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            number: default_number(),
            relativenumber: default_relativenumber(),
            tabstop: default_tabstop(),
            shiftwidth: default_shiftwidth(),
            hlsearch: default_hlsearch(),
            ignorecase: default_ignorecase(),
            smartcase: default_smartcase(),
            scrolloff: default_scrolloff(),
            menu: default_menu(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if !path.exists() {
            let _ = fs::write(&path, DEFAULT_CONFIG);
            return Config::default();
        }
        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("rvim: invalid config: {}", e);
                Config::default()
            }),
            Err(e) => {
                eprintln!("rvim: cannot read config: {}", e);
                Config::default()
            }
        }
    }

    pub fn set(&mut self, key: &str, value: Option<&str>) -> Result<(), String> {
        match key {
            "number" | "nu" => self.number = parse_bool(value)?,
            "nonumber" | "nonu" => self.number = false,
            "relativenumber" | "rnu" => self.relativenumber = parse_bool(value)?,
            "norelativenumber" | "nornu" => self.relativenumber = false,
            "hlsearch" | "hls" => self.hlsearch = parse_bool(value)?,
            "nohlsearch" | "nohls" => self.hlsearch = false,
            "ignorecase" | "ic" => self.ignorecase = parse_bool(value)?,
            "noignorecase" | "noic" => self.ignorecase = false,
            "smartcase" | "scs" => self.smartcase = parse_bool(value)?,
            "nosmartcase" | "noscs" => self.smartcase = false,
            "tabstop" | "ts" => self.tabstop = parse_usize(value)?,
            "shiftwidth" | "sw" => self.shiftwidth = parse_usize(value)?,
            "scrolloff" | "so" => self.scrolloff = parse_usize(value)?,
            "menu" => self.menu = parse_bool(value)?,
            "nomenu" => self.menu = false,
            _ => return Err(format!("Unknown option: {}", key)),
        }
        Ok(())
    }

    pub fn get(&self) -> Vec<String> {
        vec![
            format!("number={}", if self.number { "true" } else { "false" }),
            format!(
                "relativenumber={}",
                if self.relativenumber { "true" } else { "false" }
            ),
            format!("tabstop={}", self.tabstop),
            format!("shiftwidth={}", self.shiftwidth),
            format!("hlsearch={}", if self.hlsearch { "true" } else { "false" }),
            format!(
                "ignorecase={}",
                if self.ignorecase { "true" } else { "false" }
            ),
            format!(
                "smartcase={}",
                if self.smartcase { "true" } else { "false" }
            ),
            format!("scrolloff={}", self.scrolloff),
            format!("menu={}", if self.menu { "true" } else { "false" }),
        ]
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".config")
        .join("rvim")
        .join("config.toml")
}

fn parse_bool(value: Option<&str>) -> Result<bool, String> {
    match value {
        Some("true") | Some("1") | Some("yes") => Ok(true),
        Some("false") | Some("0") | Some("no") => Ok(false),
        Some(v) => Err(format!("Invalid bool: {}", v)),
        None => Ok(true),
    }
}

const DEFAULT_CONFIG: &str = "\
# rvim configuration file.
#
# At runtime:   :set <option>=<value>
#   boolean toggle:  :set <option>     (sets to true)
#   boolean disable: :set no<option>   (sets to false)
#

# === Display ===

# Show line numbers  |  :set number  :set nonu
number = true

# Show relative line numbers  |  :set relativenumber  :set nornu
relativenumber = false

# Show the menu bar at the top of the screen  |  :set menu  :set nomenu
menu = true

# === Search ===

# Highlight all search matches  |  :set hlsearch  :set nohls
hlsearch = true

# Ignore case in search patterns  |  :set ignorecase  :set noic
ignorecase = false

# Override ignorecase when pattern has uppercase letters  |  :set smartcase  :set noscs
smartcase = true

# === Indentation ===

# Number of spaces a <Tab> counts for  |  :set tabstop=4  :set ts=2
tabstop = 8

# Number of spaces for each indentation step (<<, >>)  |  :set shiftwidth=4  :set sw=2
shiftwidth = 8

# === Scrolling ===

# Minimum screen lines to keep above/below the cursor  |  :set scrolloff=5  :set so=3
scrolloff = 0
";

fn parse_usize(value: Option<&str>) -> Result<usize, String> {
    match value {
        Some(v) => v.parse().map_err(|_| format!("Invalid number: {}", v)),
        None => Err("Expected value".into()),
    }
}
