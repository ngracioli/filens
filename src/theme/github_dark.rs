use super::{Theme, rgb, syntax};

// https://primer.style/product/getting-started/react/theme-reference/
// https://github.com/primer/github-vscode-theme/blob/main/src/colors.js
// https://github.com/primer/github-vscode-theme/blob/main/src/theme.js
pub(super) const PALETTE: Theme = Theme {
    name: "GitHub Dark",
    surface: rgb(0x0d1117),
    raised: rgb(0x161b22),
    border: rgb(0x30363d),
    text: rgb(0xe6edf3),
    muted: rgb(0x7d8590),
    accent: rgb(0x58a6ff),
    directory: rgb(0x79c0ff),
    syntax: syntax([
        0x8b949e, 0xa5d6ff, 0xff7b72, 0x79c0ff, 0xd2a8ff, 0xffa657, 0x7ee787, 0x79c0ff,
    ]),
};
