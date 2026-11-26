use super::{Theme, rgb};

// https://github.com/microsoft/vscode/blob/main/extensions/theme-monokai/themes/monokai-color-theme.json
pub(super) const PALETTE: Theme = Theme {
    name: "Monokai",
    surface: rgb(0x272822),
    raised: rgb(0x1e1f1c),
    border: rgb(0x414339),
    text: rgb(0xf8f8f2),
    muted: rgb(0xcfcfc2),
    accent: rgb(0xa6e22e),
    directory: rgb(0x66d9ef),
};
