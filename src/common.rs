use std::fmt;

// Emoji used during report output
pub enum Emoji {
    Arrow,
    Cross,
    Tick,
    Warning,
}

impl fmt::Display for Emoji {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let symbol = match *self {
        Self::Arrow => "❯",
        Self::Cross => "✖",
        Self::Tick => "✔",
        Self::Warning => "⚠️",
    };

    write!(f, "{}", symbol)
  }
}

// A boolean wrapper than you can Display into unicode symbols
pub struct EmojiBool(bool);

impl fmt::Display for EmojiBool {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let item = if self.0 {
      Emoji::Tick
    }
    else {
      Emoji::Cross
    };

    item.fmt(f)
  }
}

impl From<bool> for EmojiBool {
  fn from(item: bool) -> Self {
    Self(item)
  }
}
