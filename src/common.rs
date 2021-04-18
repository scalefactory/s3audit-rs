use std::fmt;

// Emoji used during report output
pub enum Emoji {
    Arrow,
    Cross,
    Info,
    Tick,
    Warning,
}

impl fmt::Display for Emoji {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let symbol = match *self {
        Self::Arrow => "❯",
        Self::Cross => "✖",
        Self::Info => "🛈",
        Self::Tick => "✔",
        Self::Warning => "⚠️ ",
    };

    write!(f, "{}", symbol)
  }
}

impl From<bool> for Emoji {
    fn from(item: bool) -> Self {
        match item {
            true => Self::Tick,
            _    => Self::Cross,
        }
    }
}
