mod ayu_mirage;
mod catppuccin_mocha;
mod dracula;
mod filens_mono;
mod github_dark;
mod gruvbox_dark;
mod material_palenight;
mod monokai;
mod night_owl;
mod nord;
mod one_dark;
mod rose_pine;
mod solarized_dark;
mod tokyo_night;
mod vesper;

use ratatui::style::Color;

pub(crate) struct Theme {
    pub(crate) name: &'static str,
    pub(crate) surface: Color,
    pub(crate) raised: Color,
    pub(crate) border: Color,
    pub(crate) text: Color,
    pub(crate) muted: Color,
    pub(crate) accent: Color,
    pub(crate) directory: Color,
}

pub(crate) const THEMES: &[Theme] = &[
    dracula::PALETTE,
    one_dark::PALETTE,
    github_dark::PALETTE,
    material_palenight::PALETTE,
    vesper::PALETTE,
    nord::PALETTE,
    tokyo_night::PALETTE,
    night_owl::PALETTE,
    catppuccin_mocha::PALETTE,
    rose_pine::PALETTE,
    ayu_mirage::PALETTE,
    gruvbox_dark::PALETTE,
    monokai::PALETTE,
    solarized_dark::PALETTE,
    filens_mono::PALETTE,
];

const fn rgb(hex: u32) -> Color {
    let [_, red, green, blue] = hex.to_be_bytes();
    Color::Rgb(red, green, blue)
}
