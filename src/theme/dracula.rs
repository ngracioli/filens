use super::{Theme, rgb, syntax};

// https://draculatheme.com/contribute#color-palette
// https://github.com/dracula/visual-studio-code/blob/main/src/dracula.yml
pub(super) const PALETTE: Theme = Theme {
    name: "Dracula",
    surface: rgb(0x282a36),
    raised: rgb(0x44475a),
    border: rgb(0x6272a4),
    text: rgb(0xf8f8f2),
    muted: rgb(0x6272a4),
    accent: rgb(0xbd93f9),
    directory: rgb(0x8be9fd),
    syntax: syntax([
        0x6272a4, 0xf1fa8c, 0xff79c6, 0xbd93f9, 0x50fa7b, 0x8be9fd, 0xff79c6, 0xbd93f9,
    ]),
};
