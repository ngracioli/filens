use super::{Theme, rgb};

// https://material-theme.com/docs/reference/color-palette/
pub(super) const PALETTE: Theme = Theme {
    name: "Material Theme Palenight",
    surface: rgb(0x292d3e),
    raised: rgb(0x34324a),
    border: rgb(0x2b2a3e),
    text: rgb(0xa6accd),
    muted: rgb(0x676e95),
    accent: rgb(0x82aaff),
    directory: rgb(0xffcb6b),
};
