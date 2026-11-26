use super::{Theme, rgb, syntax};

// https://github.com/raunofreiberg/vesper/blob/main/themes/Vesper-dark-color-theme.json
pub(super) const PALETTE: Theme = Theme {
    name: "Vesper",
    surface: rgb(0x101010),
    raised: rgb(0x161616),
    border: rgb(0x282828),
    text: rgb(0xffffff),
    muted: rgb(0xa0a0a0),
    accent: rgb(0xffc799),
    directory: rgb(0x99ffe4),
    syntax: syntax([
        0x575757, 0x99ffe4, 0xa0a0a0, 0xffc799, 0xffc799, 0xffc799, 0xffc799, 0xffc799,
    ]),
};
