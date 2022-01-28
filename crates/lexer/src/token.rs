//! Token

use std::ops::Range;

use crate::kind::Kind;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Token {
    /// Token Kind
    kind: Kind,

    /// Offset of token in source
    offset: usize,

    /// Length of token
    len: usize,
}

impl Token {
    #[must_use]
    pub const fn new(kind: Kind, offset: usize, len: usize) -> Self {
        Self { kind, offset, len }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub const fn range(&self) -> Range<usize> {
        self.offset..(self.offset + self.len)
    }

    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.kind == Kind::Unknown
    }
}
