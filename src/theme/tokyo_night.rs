use super::{Theme, rgb};

// https://github.com/enkia/tokyo-night-vscode-theme/blob/master/themes/tokyo-night-color-theme.json
pub(super) const PALETTE: Theme = Theme {
    name: "Tokyo Night",
    surface: rgb(0x1a1b26),
    raised: rgb(0x16161e),
    border: rgb(0x363b54),
    text: rgb(0xa9b1d6),
    muted: rgb(0x9aa5ce),
    accent: rgb(0x7aa2f7),
    directory: rgb(0x7dcfff),
};
