use super::{Theme, rgb};

// https://primer.style/product/getting-started/react/theme-reference/
// https://github.com/primer/github-vscode-theme/blob/main/src/colors.js
pub(super) const PALETTE: Theme = Theme {
    name: "GitHub Dark",
    surface: rgb(0x0d1117),
    raised: rgb(0x161b22),
    border: rgb(0x30363d),
    text: rgb(0xe6edf3),
    muted: rgb(0x7d8590),
    accent: rgb(0x58a6ff),
    directory: rgb(0x79c0ff),
};
