use inno::header::AutoBool;

pub trait Emoji {
    fn emoji(&self) -> &'static str;
}

impl Emoji for bool {
    fn emoji(&self) -> &'static str {
        if *self { "✅" } else { "❌" }
    }
}

impl Emoji for AutoBool {
    fn emoji(&self) -> &'static str {
        match self {
            Self::Auto => "🔄",
            Self::No => "❌",
            Self::Yes => "✅",
        }
    }
}
