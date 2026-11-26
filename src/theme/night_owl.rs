use super::{Theme, rgb};

// https://github.com/sdras/night-owl-vscode-theme/blob/main/themes/Night%20Owl-color-theme.json
pub(super) const PALETTE: Theme = Theme {
    name: "Night Owl",
    surface: rgb(0x011627),
    raised: rgb(0x0b253a),
    border: rgb(0x122d42),
    text: rgb(0xd6deeb),
    muted: rgb(0x80a4c2),
    accent: rgb(0x82aaff),
    directory: rgb(0x7fdbca),
};
