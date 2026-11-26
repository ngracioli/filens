use super::{Theme, rgb, syntax};

pub(super) const PALETTE: Theme = Theme {
    name: "Filens Mono",
    surface: rgb(0x242424),
    raised: rgb(0x2c2c2c),
    border: rgb(0x474747),
    text: rgb(0xc8c8c8),
    muted: rgb(0xa0a0a0),
    accent: rgb(0x939393),
    directory: rgb(0xb8b8b8),
    syntax: syntax([
        0xa0a0a0, 0xd8d8d8, 0xefefef, 0xb8b8b8, 0xe0e0e0, 0xb8b8b8, 0xe0e0e0, 0xefefef,
    ]),
};
