use super::{Theme, rgb, syntax};

pub(super) const PALETTE: Theme = Theme {
    name: "Filens Mono",
    surface: rgb(0x1c1c1c),
    raised: rgb(0x282828),
    border: rgb(0x555555),
    text: rgb(0xdedede),
    muted: rgb(0xa0a0a0),
    accent: rgb(0xb0b0b0),
    directory: rgb(0xc4c4c4),
    syntax: syntax([
        0xa0a0a0, 0xcacaca, 0xf3f3f3, 0xb8b8b8, 0xe9e9e9, 0xb8b8b8, 0xe9e9e9, 0xf3f3f3,
    ]),
};
