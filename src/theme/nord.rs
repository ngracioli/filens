use super::{Theme, rgb, syntax};

// https://www.nordtheme.com/docs/colors-and-palettes/
// https://github.com/nordtheme/visual-studio-code/blob/develop/themes/nord-color-theme.json
pub(super) const PALETTE: Theme = Theme {
    name: "Nord",
    surface: rgb(0x2e3440),
    raised: rgb(0x3b4252),
    border: rgb(0x4c566a),
    text: rgb(0xeceff4),
    muted: rgb(0xd8dee9),
    accent: rgb(0x88c0d0),
    directory: rgb(0xebcb8b),
    syntax: syntax([
        0x616e88, 0xa3be8c, 0x81a1c1, 0xb48ead, 0x88c0d0, 0x8fbcbb, 0x81a1c1, 0x88c0d0,
    ]),
};
