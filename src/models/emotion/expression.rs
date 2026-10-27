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
    /// Active random moods when companion is awake
    pub const RANDOM_MOODS: [EmoExpression; 15] = [
        Self::Normal,
        Self::Relaxed,
        Self::Winking,
        Self::Angry,
        Self::Rage,
        Self::AngryBlush,
        Self::Crying,
        Self::FlatSlit,
        Self::Happy,
        Self::Squint,
        Self::Love,
        Self::Sparkle,
        Self::Surprised,
        Self::Starry,
        Self::Dizzy,
    ];
}
