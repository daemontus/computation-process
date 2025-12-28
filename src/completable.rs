use cancel_this::Cancelled;
use std::fmt::{Display, Formatter};

/// The error type returned by an algorithm when the result is not (yet) available.
///
/// The result can be unavailable because the computation was canceled ([`Cancelled`]) or because
/// the algorithm has not finished computing but reached one of its pre-defined suspend points.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Incomplete {
    Suspended,
    Cancelled(Cancelled),
}

/// A [`Completable`] result is a value eventually computed by an algorithm where
/// the computation can be [`Incomplete`] when the value is polled.
pub type Completable<T> = Result<T, Incomplete>;

impl From<Cancelled> for Incomplete {
    fn from(value: Cancelled) -> Self {
        Incomplete::Cancelled(value)
    }
}

impl Display for Incomplete {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Incomplete::Suspended => write!(f, "Operation suspended"),
            Incomplete::Cancelled(c) => write!(f, "{}", c),
        }
    }
}

impl std::error::Error for Incomplete {}
