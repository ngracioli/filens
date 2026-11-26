use super::{Theme, rgb};

// https://github.com/rose-pine/vscode/blob/main/themes/rose-pine-color-theme.json
pub(super) const PALETTE: Theme = Theme {
    name: "Rosé Pine",
    surface: rgb(0x191724),
    raised: rgb(0x1f1d2e),
    border: rgb(0x26233a),
    text: rgb(0xe0def4),
    muted: rgb(0x908caa),
    accent: rgb(0xebbcba),
    directory: rgb(0x9ccfd8),
};
