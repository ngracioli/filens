use super::{Theme, rgb, syntax};

// https://ethanschoonover.com/solarized/
// https://github.com/microsoft/vscode/blob/main/extensions/theme-solarized-dark/themes/solarized-dark-color-theme.json
pub(super) const PALETTE: Theme = Theme {
    name: "Solarized Dark",
    surface: rgb(0x002b36),
    raised: rgb(0x073642),
    border: rgb(0x586e75),
    text: rgb(0x93a1a1),
    muted: rgb(0x839496),
    accent: rgb(0x268bd2),
    directory: rgb(0x2aa198),
    syntax: syntax([
        0x586e75, 0x2aa198, 0x859900, 0xd33682, 0x268bd2, 0xcb4b16, 0x268bd2, 0x268bd2,
    ]),
};
