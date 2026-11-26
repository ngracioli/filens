use super::{Theme, rgb};

// https://ethanschoonover.com/solarized/
pub(super) const PALETTE: Theme = Theme {
    name: "Solarized Dark",
    surface: rgb(0x002b36),
    raised: rgb(0x073642),
    border: rgb(0x586e75),
    text: rgb(0x93a1a1),
    muted: rgb(0x839496),
    accent: rgb(0x268bd2),
    directory: rgb(0x2aa198),
};
