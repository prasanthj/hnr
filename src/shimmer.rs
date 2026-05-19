use std::sync::OnceLock;
use std::time::{Duration, Instant};

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

static START: OnceLock<Instant> = OnceLock::new();

fn process_elapsed() -> Duration {
    START.get_or_init(Instant::now).elapsed()
}

fn blend(fg: (u8, u8, u8), bg: (u8, u8, u8), alpha: f32) -> (u8, u8, u8) {
    let r = (fg.0 as f32 * alpha + bg.0 as f32 * (1.0 - alpha)) as u8;
    let g = (fg.1 as f32 * alpha + bg.1 as f32 * (1.0 - alpha)) as u8;
    let b = (fg.2 as f32 * alpha + bg.2 as f32 * (1.0 - alpha)) as u8;
    (r, g, b)
}

fn shimmer_spans(text: &str, base: (u8, u8, u8), highlight: (u8, u8, u8)) -> Vec<Span<'static>> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let padding = 8usize;
    let period = chars.len() + padding * 2;
    let pos = ((process_elapsed().as_secs_f32() % 2.0) / 2.0 * period as f32) as isize;
    let band_half = 5.0f32;

    chars.iter().enumerate().map(|(i, ch)| {
        let dist = ((i + padding) as isize - pos).abs() as f32;
        let t = if dist <= band_half {
            0.5 * (1.0 + (std::f32::consts::PI * dist / band_half).cos())
        } else {
            0.0
        };
        let (r, g, b) = blend(highlight, base, t * 0.9);
        let bold = if t > 0.6 { Modifier::BOLD } else { Modifier::empty() };
        Span::styled(ch.to_string(), Style::default().fg(Color::Rgb(r, g, b)).add_modifier(bold))
    }).collect()
}

// Orange beam: in-progress operations
const BASE_ORANGE: (u8, u8, u8) = (100, 50, 0);
const HIGH_ORANGE: (u8, u8, u8) = (255, 180, 80);

/// An active progress indicator. Renders with an animated shimmer beam and
/// trailing dots that cycle every second (.  →  ..  →  ...).
pub struct Progress {
    pub label: String,
    started_at: Instant,
}

impl Progress {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), started_at: Instant::now() }
    }

    pub fn spans(&self) -> Vec<Span<'static>> {
        let dots = (self.started_at.elapsed().as_secs() % 3) + 1;
        // Pad to 3 dots always so the widget width never changes.
        let text = format!("{}{}{}", self.label, ".".repeat(dots as usize), " ".repeat(3 - dots as usize));
        shimmer_spans(&text, BASE_ORANGE, HIGH_ORANGE)
    }
}
