//! Mako Group colour palette as ratatui Color::Rgb constants.

use ratatui::style::Color;

pub struct MakoColors;

#[allow(dead_code)]
impl MakoColors {
    // Primary palette
    pub const NAVY: Color = Color::Rgb(14, 30, 63); // #0E1E3F
    pub const BLUE: Color = Color::Rgb(84, 113, 223); // #5471DF
    pub const GOLD: Color = Color::Rgb(178, 140, 84); // #B28C54
    pub const LIGHT_BLUE: Color = Color::Rgb(219, 225, 245); // #DBE1F5
    pub const SUCCESS: Color = Color::Rgb(44, 95, 45); // #2C5F2D
    pub const ERROR: Color = Color::Rgb(184, 80, 66); // #B85042

    // Surfaces
    pub const BG: Color = Color::Rgb(244, 246, 251); // #F4F6FB
    pub const SURFACE: Color = Color::Rgb(255, 255, 255); // #FFFFFF
    pub const BORDER: Color = Color::Rgb(209, 217, 232); // #D1D9E8

    // Text
    pub const TEXT_PRIMARY: Color = Color::Rgb(45, 55, 72); // #2D3748
    pub const TEXT_SECONDARY: Color = Color::Rgb(107, 122, 153); // #6B7A99
    pub const TEXT_ON_NAVY: Color = Color::Rgb(255, 255, 255);
}
