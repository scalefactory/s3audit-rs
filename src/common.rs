use colored::Colorize;
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
        Self::Arrow => "❯".yellow(),
        Self::Cross => "✖".red(),
        Self::Info => "🛈".cyan(),
        Self::Tick => "✔".green(),
        Self::Warning => "⚠️ ".cyan(),
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
