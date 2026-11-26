use super::{Theme, rgb, syntax};

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
    syntax: syntax([
        0x88846f, 0xe6db74, 0xf92672, 0xae81ff, 0xa6e22e, 0xa6e22e, 0xf92672, 0xa6e22e,
    ]),
};
