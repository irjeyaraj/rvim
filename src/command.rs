// Copyright (c) 2026 Immmanuel Jeyaraj <irj@sefier.com>. MIT License.

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Quit,
    ForceQuit,
    Write,
    ForceWrite,
    WriteQuit,
    ForceWriteQuit,
    WriteAll,
    QuitAll,
    ForceQuitAll,
    Open(String),
    Set(String, Option<String>),
    NoHLSearch,
    Help,
    ShowOptions,
    New,
    Vnew,
    Only,
    BufferNext,
    BufferPrev,
    BufferList,
}

pub fn parse(input: &str) -> Result<Command, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("".into());
    }

    let (cmd, rest) = if let Some(pos) = input.find([' ', '=']) {
        let (c, r) = input.split_at(pos);
        let r = r.trim();
        (c, r.strip_prefix('=').unwrap_or(r))
    } else {
        (input, "")
    };

    match cmd {
        "q" => Ok(Command::Quit),
        "q!" => Ok(Command::ForceQuit),
        "quit" => Ok(Command::Quit),
        "quit!" => Ok(Command::ForceQuit),
        "w" => Ok(Command::Write),
        "w!" => Ok(Command::ForceWrite),
        "write" => Ok(Command::Write),
        "wq" => Ok(Command::WriteQuit),
        "wq!" => Ok(Command::ForceWriteQuit),
        "wa" | "wall" => Ok(Command::WriteAll),
        "qa" | "qall" => Ok(Command::QuitAll),
        "qa!" | "qall!" => Ok(Command::ForceQuitAll),
        "e" | "edit" => {
            let file = rest.trim();
            if file.is_empty() {
                Err("No filename".into())
            } else {
                Ok(Command::Open(file.to_string()))
            }
        }
        "set" => {
            let eq_pos = rest.find('=');
            let (key, value) = if let Some(pos) = eq_pos {
                let (k, v) = rest.split_at(pos);
                (k.trim().to_string(), Some(v[1..].trim().to_string()))
            } else {
                (rest.trim().to_string(), None)
            };
            if key.is_empty() {
                Ok(Command::ShowOptions)
            } else {
                Ok(Command::Set(key, value))
            }
        }
        "nohlsearch" | "noh" | "nohl" => Ok(Command::NoHLSearch),
        "help" | "h" => Ok(Command::Help),
        "new" => Ok(Command::New),
        "vnew" | "vne" => Ok(Command::Vnew),
        "only" => Ok(Command::Only),
        "bn" | "bnext" => Ok(Command::BufferNext),
        "bp" | "bprev" | "bprevious" => Ok(Command::BufferPrev),
        "ls" | "buffers" => Ok(Command::BufferList),
        _ => Err(format!("Unknown command: {}", cmd)),
    }
}
