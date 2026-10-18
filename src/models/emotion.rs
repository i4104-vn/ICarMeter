//! Expressions understood by the dashboard companion.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmoExpression {
    Normal,
    Relaxed,
    Winking,
    Angry,
    Rage,
    Sleepy,
    Sleeping,
    AngryBlush,
    Crying,
    FlatSlit,
    Happy,
    Squint,
    Love,
    Sparkle,
    Surprised,
    Starry,
    Dizzy,
    Atlas(u8),
}

impl EmoExpression {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "relaxed" | "chill" => Some(Self::Relaxed),
            "wink" | "winking" => Some(Self::Winking),
            "angry" | "tucgian" | "gian" => Some(Self::Angry),
            "rage" | "fire" | "fiery" => Some(Self::Rage),
            "sleepy" | "drowsy" | "limdim" | "yawn" => Some(Self::Sleepy),
            "sleeping" | "asleep" | "sleep" | "ngu" => Some(Self::Sleeping),
            "angryblush" | "blush" | "tsundere" => Some(Self::AngryBlush),
            "crying" | "cry" | "khoc" => Some(Self::Crying),
            "flatslit" | "deadpan" | "slit" => Some(Self::FlatSlit),
            "happy" | "vui" | "smile" => Some(Self::Happy),
            "squint" | "playful" => Some(Self::Squint),
            "love" | "heart" => Some(Self::Love),
            "sparkle" | "shiny" => Some(Self::Sparkle),
            "surprised" | "surprise" => Some(Self::Surprised),
            "starry" | "star" | "starstruck" => Some(Self::Starry),
            "dizzy" | "confused" => Some(Self::Dizzy),
            _ => None,
        }
    }
}
