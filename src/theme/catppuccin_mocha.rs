use super::{Theme, rgb, syntax};

// https://catppuccin.com/palette/#mocha
// https://github.com/catppuccin/catppuccin/blob/main/docs/style-guide.md
pub(super) const PALETTE: Theme = Theme {
    name: "Catppuccin Mocha",
    surface: rgb(0x1e1e2e),
    raised: rgb(0x181825),
    border: rgb(0x45475a),
    text: rgb(0xcdd6f4),
    muted: rgb(0xa6adc8),
    accent: rgb(0xcba6f7),
    directory: rgb(0x89b4fa),
    syntax: syntax([
        0x9399b2, 0xa6e3a1, 0xcba6f7, 0xfab387, 0x89b4fa, 0xf9e2af, 0xcba6f7, 0xf38ba8,
    ]),
};
