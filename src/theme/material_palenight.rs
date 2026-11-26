use super::{Theme, rgb, syntax};

// https://material-theme.com/docs/reference/color-palette/
// https://github.com/Nsttt/vsc-material-theme/blob/244a81c01934cbe78490b98d80c85b8f27d926a8/scripts/generator/color-set.ts
// https://github.com/Nsttt/vsc-material-theme/blob/244a81c01934cbe78490b98d80c85b8f27d926a8/scripts/generator/settings/specific/palenight.ts
pub(super) const PALETTE: Theme = Theme {
    name: "Material Theme Palenight",
    surface: rgb(0x292d3e),
    raised: rgb(0x34324a),
    border: rgb(0x2b2a3e),
    text: rgb(0xa6accd),
    muted: rgb(0x676e95),
    accent: rgb(0x82aaff),
    directory: rgb(0xffcb6b),
    syntax: syntax([
        0x676e95, 0xc3e88d, 0x89ddff, 0xf78c6c, 0x82aaff, 0xffcb6b, 0xf07178, 0x82aaff,
    ]),
};
