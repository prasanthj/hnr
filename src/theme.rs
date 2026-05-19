use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub const BG: Color          = Color::Rgb(0x1a, 0x1b, 0x1e);
    pub const PRIMARY: Color     = Color::Rgb(0xe6, 0xe1, 0xd7);
    pub const SECONDARY: Color   = Color::Rgb(0x6b, 0x6b, 0x6b);
    pub const DIM: Color         = Color::Rgb(0x3f, 0x3f, 0x46);
    pub const ACCENT: Color      = Color::Rgb(0xff, 0x8c, 0x42);
    pub const ACCENT_SOFT: Color = Color::Rgb(0xd4, 0xa5, 0x74);
    pub const SELECT_BG: Color   = Color::Rgb(0x2a, 0x25, 0x20);

    pub fn primary() -> Style   { Style::default().fg(Self::PRIMARY) }
    pub fn secondary() -> Style { Style::default().fg(Self::SECONDARY) }
    pub fn dim() -> Style       { Style::default().fg(Self::DIM) }
    pub fn accent() -> Style    { Style::default().fg(Self::ACCENT) }
    pub fn accent_soft() -> Style { Style::default().fg(Self::ACCENT_SOFT) }
    pub fn title() -> Style     { Style::default().fg(Self::PRIMARY).add_modifier(Modifier::BOLD) }
}
