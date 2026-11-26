use super::{Theme, rgb};

// https://github.com/ayu-theme/vscode-ayu/blob/master/ayu-mirage.json
pub(super) const PALETTE: Theme = Theme {
    name: "Ayu Mirage",
    surface: rgb(0x242936),
    raised: rgb(0x1f2430),
    border: rgb(0x171b24),
    text: rgb(0xffffff),
    muted: rgb(0xc7c7c7),
    accent: rgb(0xffcc66),
    directory: rgb(0x73d0ff),
};
