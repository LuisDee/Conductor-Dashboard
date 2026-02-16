//! Switchable theme system with semantic color roles.

use ratatui::style::Color;

/// A complete color theme with semantic role names.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub name: &'static str,

    // Bar / chrome
    pub bar_bg: Color,
    pub text_on_bar: Color,

    // Accent
    pub accent: Color,
    #[allow(dead_code)]
    pub accent_light: Color,

    // Semantic
    pub warning: Color,
    pub success: Color,
    pub error: Color,

    // Surfaces
    #[allow(dead_code)]
    pub bg: Color,
    pub surface: Color,
    pub border: Color,

    // Text
    pub text_primary: Color,
    pub text_secondary: Color,

    // Progress bar (overridable per theme)
    pub progress_active: Color,
    pub progress_done: Color,
    pub progress_blocked: Color,
    pub progress_new: Color,
}

const ALL_THEMES: [Theme; 6] = [
    Theme::mako(),
    Theme::warm_dark(),
    Theme::midnight(),
    Theme::ember(),
    Theme::dusk(),
    Theme::light(),
];

impl Theme {
    pub const fn mako() -> Self {
        Self {
            name: "Mako",
            bar_bg: Color::Rgb(14, 30, 63),
            text_on_bar: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(84, 113, 223),
            accent_light: Color::Rgb(219, 225, 245),
            warning: Color::Rgb(178, 140, 84),
            success: Color::Rgb(44, 95, 45),
            error: Color::Rgb(184, 80, 66),
            bg: Color::Rgb(244, 246, 251),
            surface: Color::Rgb(255, 255, 255),
            border: Color::Rgb(209, 217, 232),
            text_primary: Color::Rgb(45, 55, 72),
            text_secondary: Color::Rgb(107, 122, 153),
            progress_active: Color::Rgb(84, 113, 223),
            progress_done: Color::Rgb(44, 95, 45),
            progress_blocked: Color::Rgb(178, 140, 84),
            progress_new: Color::Rgb(107, 122, 153),
        }
    }

    pub const fn warm_dark() -> Self {
        Self {
            name: "Warm Dark",
            bar_bg: Color::Rgb(30, 30, 29),
            text_on_bar: Color::Rgb(232, 230, 220),
            accent: Color::Rgb(106, 155, 204),
            accent_light: Color::Rgb(130, 176, 217),
            warning: Color::Rgb(201, 168, 76),
            success: Color::Rgb(120, 140, 93),
            error: Color::Rgb(196, 91, 91),
            bg: Color::Rgb(25, 25, 24),
            surface: Color::Rgb(37, 37, 36),
            border: Color::Rgb(51, 51, 49),
            text_primary: Color::Rgb(232, 230, 220),
            text_secondary: Color::Rgb(122, 120, 111),
            progress_active: Color::Rgb(217, 119, 87),
            progress_done: Color::Rgb(120, 140, 93),
            progress_blocked: Color::Rgb(201, 168, 76),
            progress_new: Color::Rgb(106, 155, 204),
        }
    }

    pub const fn midnight() -> Self {
        Self {
            name: "Midnight",
            bar_bg: Color::Rgb(23, 26, 33),
            text_on_bar: Color::Rgb(208, 212, 220),
            accent: Color::Rgb(123, 170, 212),
            accent_light: Color::Rgb(142, 189, 224),
            warning: Color::Rgb(201, 168, 76),
            success: Color::Rgb(125, 155, 106),
            error: Color::Rgb(196, 91, 91),
            bg: Color::Rgb(18, 20, 26),
            surface: Color::Rgb(30, 34, 48),
            border: Color::Rgb(46, 51, 64),
            text_primary: Color::Rgb(208, 212, 220),
            text_secondary: Color::Rgb(90, 98, 120),
            progress_active: Color::Rgb(217, 119, 87),
            progress_done: Color::Rgb(125, 155, 106),
            progress_blocked: Color::Rgb(201, 168, 76),
            progress_new: Color::Rgb(123, 170, 212),
        }
    }

    pub const fn ember() -> Self {
        Self {
            name: "Ember",
            bar_bg: Color::Rgb(32, 25, 22),
            text_on_bar: Color::Rgb(224, 216, 204),
            accent: Color::Rgb(106, 155, 204),
            accent_light: Color::Rgb(130, 176, 217),
            warning: Color::Rgb(201, 168, 76),
            success: Color::Rgb(138, 155, 104),
            error: Color::Rgb(196, 91, 91),
            bg: Color::Rgb(26, 20, 18),
            surface: Color::Rgb(40, 32, 28),
            border: Color::Rgb(61, 49, 43),
            text_primary: Color::Rgb(224, 216, 204),
            text_secondary: Color::Rgb(122, 107, 92),
            progress_active: Color::Rgb(217, 119, 87),
            progress_done: Color::Rgb(138, 155, 104),
            progress_blocked: Color::Rgb(201, 168, 76),
            progress_new: Color::Rgb(106, 155, 204),
        }
    }

    pub const fn dusk() -> Self {
        Self {
            name: "Dusk",
            bar_bg: Color::Rgb(51, 50, 48),
            text_on_bar: Color::Rgb(236, 233, 224),
            accent: Color::Rgb(106, 155, 204),
            accent_light: Color::Rgb(130, 176, 217),
            warning: Color::Rgb(201, 168, 76),
            success: Color::Rgb(120, 140, 93),
            error: Color::Rgb(196, 91, 91),
            bg: Color::Rgb(44, 43, 40),
            surface: Color::Rgb(58, 57, 55),
            border: Color::Rgb(78, 77, 72),
            text_primary: Color::Rgb(236, 233, 224),
            text_secondary: Color::Rgb(138, 135, 125),
            progress_active: Color::Rgb(217, 119, 87),
            progress_done: Color::Rgb(120, 140, 93),
            progress_blocked: Color::Rgb(201, 168, 76),
            progress_new: Color::Rgb(106, 155, 204),
        }
    }

    pub const fn light() -> Self {
        Self {
            name: "Light",
            bar_bg: Color::Rgb(234, 232, 224),
            text_on_bar: Color::Rgb(26, 26, 25),
            accent: Color::Rgb(74, 125, 168),
            accent_light: Color::Rgb(90, 141, 184),
            warning: Color::Rgb(154, 123, 46),
            success: Color::Rgb(93, 122, 66),
            error: Color::Rgb(184, 76, 63),
            bg: Color::Rgb(244, 243, 238),
            surface: Color::Rgb(250, 249, 245),
            border: Color::Rgb(223, 221, 213),
            text_primary: Color::Rgb(26, 26, 25),
            text_secondary: Color::Rgb(138, 135, 126),
            progress_active: Color::Rgb(193, 95, 60),
            progress_done: Color::Rgb(93, 122, 66),
            progress_blocked: Color::Rgb(154, 123, 46),
            progress_new: Color::Rgb(74, 125, 168),
        }
    }

    /// Returns all available theme presets.
    pub fn all() -> &'static [Theme] {
        &ALL_THEMES
    }

    /// Cycle to the next theme in the preset list.
    pub fn next(&self) -> Theme {
        let themes = Self::all();
        let current_idx = themes
            .iter()
            .position(|t| t.name == self.name)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % themes.len();
        themes[next_idx]
    }
}
