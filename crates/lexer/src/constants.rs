pub const ASCII_SPACES: [u8; 4] = [b' ', 9, 11, 12];

pub const UNICODE_SPACES: [char; 22] = [
    '\u{0020}', '\u{0009}', '\u{000B}', '\u{000C}', '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}',
    '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}',
    '\u{200A}', '\u{200B}', '\u{202F}', '\u{205F}', '\u{3000}', '\u{FEFF}',
];

pub const ASCII_LINE_TERMINATORS: [u8; 2] = [b'\n', b'\r'];
pub const ASCII_LINE_TERMINATORS_CHAR: [char; 2] = ['\n', '\r'];

pub const UNICODE_LINE_TERMINATORS: [char; 2] = ['\u{2028}', '\u{2029}'];
