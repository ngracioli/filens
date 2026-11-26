use super::{Theme, rgb};

// https://draculatheme.com/contribute#color-palette
pub(super) const PALETTE: Theme = Theme {
    name: "Dracula",
    surface: rgb(0x282a36),
    raised: rgb(0x44475a),
    border: rgb(0x6272a4),
    text: rgb(0xf8f8f2),
    muted: rgb(0x6272a4),
    accent: rgb(0xbd93f9),
    directory: rgb(0x8be9fd),
};
