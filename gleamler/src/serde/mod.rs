pub mod de;
pub mod error;
pub mod ser;
pub mod util;

mod atoms;

pub use de::from_term;
pub use ser::to_term;
pub use ser::to_term_with;

use crate::Term;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SerdeTerm<'a>(pub Term<'a>);

impl<'a> SerdeTerm<'a> {
    pub fn from_term(term: Term<'a>) -> Self {
        Self(term)
    }

    pub fn into_term(self) -> Term<'a> {
        self.0
    }
}
