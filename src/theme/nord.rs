use super::{Theme, rgb};

// https://www.nordtheme.com/docs/colors-and-palettes/
pub(super) const PALETTE: Theme = Theme {
    name: "Nord",
    surface: rgb(0x2e3440),
    raised: rgb(0x3b4252),
    border: rgb(0x4c566a),
    text: rgb(0xeceff4),
    muted: rgb(0xd8dee9),
    accent: rgb(0x88c0d0),
    directory: rgb(0xebcb8b),
};
