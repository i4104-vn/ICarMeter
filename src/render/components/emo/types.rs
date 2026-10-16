// Expressions used by the animated sprite companion.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmoExpression {
    // Strip 1
    Normal,
    Relaxed,
    Winking,

    // Strip 2
    Angry,
    Rage,

    // Strip 3 (Sleep Progression)
    Sleepy,
    Sleeping,

    // Strip 4
    AngryBlush,
    Crying,
    FlatSlit,

    // Strip 5
    Happy,
    Squint,

    // Additional reactions in the supplied expression set
    Love,
    Sparkle,
    Surprised,
    Starry,
    Dizzy,

    /// A cell from the attached 7-by-6 expression sheet, selected automatically.
    Atlas(u8),
}

impl EmoExpression {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Strip 1
            "normal" => Some(Self::Normal),
            "relaxed" | "chill" => Some(Self::Relaxed),
            "wink" | "winking" => Some(Self::Winking),

            // Strip 2
            "angry" | "tucgian" | "gian" => Some(Self::Angry),
            "rage" | "fire" | "fiery" => Some(Self::Rage),

            // Strip 3
            "sleepy" | "drowsy" | "limdim" | "yawn" => Some(Self::Sleepy),
            "sleeping" | "asleep" | "sleep" | "ngu" => Some(Self::Sleeping),

            // Strip 4
            "angryblush" | "blush" | "tsundere" => Some(Self::AngryBlush),
            "crying" | "cry" | "khoc" => Some(Self::Crying),
            "flatslit" | "deadpan" | "slit" => Some(Self::FlatSlit),

            // Strip 5
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
