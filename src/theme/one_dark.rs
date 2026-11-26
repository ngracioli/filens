use super::{Theme, rgb, syntax};

// https://github.com/atom/one-dark-syntax/blob/master/styles/colors.less
// https://github.com/atom/one-dark-ui/blob/master/styles/ui-variables-custom.less
// https://github.com/atom/one-dark-syntax/blob/master/styles/syntax/_base.less
pub(super) const PALETTE: Theme = Theme {
    name: "One Dark",
    surface: rgb(0x282c34),
    raised: rgb(0x21252b),
    border: rgb(0x181a1f),
    text: rgb(0xabb2bf),
    muted: rgb(0x828997),
    accent: rgb(0x61afef),
    directory: rgb(0xe5c07b),
    syntax: syntax([
        0x5c6370, 0x98c379, 0xc678dd, 0xd19a66, 0x61afef, 0xe5c07b, 0xe06c75, 0x61afef,
    ]),
};
