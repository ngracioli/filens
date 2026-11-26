use super::{Theme, rgb};

// https://github.com/morhetz/gruvbox/blob/master/colors/gruvbox.vim
pub(super) const PALETTE: Theme = Theme {
    name: "Gruvbox Dark",
    surface: rgb(0x282828),
    raised: rgb(0x3c3836),
    border: rgb(0x504945),
    text: rgb(0xebdbb2),
    muted: rgb(0xa89984),
    accent: rgb(0xfabd2f),
    directory: rgb(0x8ec07c),
};
