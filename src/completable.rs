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

#[cfg(test)]
mod tests {
    use super::*;
    use cancel_this::Cancelled;

    #[test]
    fn test_incomplete_suspended() {
        let incomplete = Incomplete::Suspended;
        assert_eq!(incomplete, Incomplete::Suspended);
        assert_eq!(format!("{}", incomplete), "Operation suspended");
    }

    #[test]
    fn test_incomplete_cancelled() {
        let cancelled = Cancelled::default();
        let incomplete = Incomplete::Cancelled(cancelled.clone());
        assert_eq!(incomplete.clone(), Incomplete::Cancelled(cancelled.clone()));
        // Canceled's Display format may vary, so just check it's not empty
        let display_str = format!("{}", incomplete);
        assert!(!display_str.is_empty());
        // Verify it's not the Suspended message
        assert_ne!(display_str, "Operation suspended");
    }

    #[test]
    fn test_from_cancelled() {
        let cancelled = Cancelled::default();
        let incomplete: Incomplete = cancelled.clone().into();
        assert_eq!(incomplete, Incomplete::Cancelled(cancelled));
    }

    #[test]
    fn test_completable_ok() {
        let result: Completable<i32> = Ok(42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_completable_err_suspended() {
        let result: Completable<i32> = Err(Incomplete::Suspended);
        assert_eq!(result, Err(Incomplete::Suspended));
    }

    #[test]
    fn test_completable_err_cancelled() {
        let cancelled = Cancelled::default();
        let result: Completable<i32> = Err(Incomplete::Cancelled(cancelled.clone()));
        assert_eq!(result, Err(Incomplete::Cancelled(cancelled)));
    }

    #[test]
    fn test_incomplete_debug() {
        let incomplete = Incomplete::Suspended;
        let debug_str = format!("{:?}", incomplete);
        assert!(debug_str.contains("Suspended"));
    }

    #[test]
    fn test_incomplete_clone() {
        let incomplete1 = Incomplete::Suspended;
        let incomplete2 = incomplete1.clone();
        assert_eq!(incomplete1, incomplete2);
    }

    #[test]
    fn test_incomplete_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Incomplete::Suspended);
        assert!(set.contains(&Incomplete::Suspended));
    }
}
