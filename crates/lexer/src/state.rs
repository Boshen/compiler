use crate::kind::Kind;
#[allow(clippy::enum_glob_use)]
use crate::kind::Kind::*;

pub struct State {
    /// are we at a lhs expression
    expr: bool,
}

impl State {
    pub const fn new() -> Self {
        Self { expr: true }
    }

    pub fn update(&mut self, kind: &Kind) {
        if !matches!(kind, WhiteSpace | LineTerminator) {
            self.expr = kind.at_expr();
        }
    }

    pub const fn allow_read_regex(&self) -> bool {
        self.expr
    }
}
