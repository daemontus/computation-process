use crate::{Completable, Computable};

/// A trivial [`Computable`] that immediately returns a pre-computed value.
///
/// This is useful for wrapping an already-computed value in the [`Computable`] interface,
/// allowing it to be used in contexts that expect a computation.
///
/// After the value is returned once, subsequent calls to [`Computable::try_compute`] will
/// return [`Incomplete::Exhausted`](crate::Incomplete::Exhausted).
///
/// # Example
///
/// ```rust
/// use computation_process::{ComputableIdentity, Computable};
///
/// let mut identity: ComputableIdentity<i32> = 42.into();
/// assert_eq!(identity.try_compute(), Ok(42));
/// ```
#[derive(Debug)]
pub struct ComputableIdentity<T> {
    value: Option<T>,
}

impl<T> From<T> for ComputableIdentity<T> {
    fn from(value: T) -> Self {
        ComputableIdentity { value: Some(value) }
    }
}

impl<T> Computable<T> for ComputableIdentity<T> {
    fn try_compute(&mut self) -> Completable<T> {
        if let Some(result) = self.value.take() {
            Ok(result)
        } else {
            Err(crate::Incomplete::Exhausted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Computable;

    #[test]
    fn test_computable_identity_from() {
        let identity: ComputableIdentity<i32> = 42.into();
        let mut identity = identity;
        let result = identity.try_compute().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_computable_identity_compute() {
        let mut identity: ComputableIdentity<String> = "hello".to_string().into();
        let result = identity.compute().unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_computable_identity_exhausted() {
        let mut identity: ComputableIdentity<i32> = 42.into();
        let _ = identity.try_compute().unwrap();
        // The second call should return Exhausted
        assert_eq!(identity.try_compute(), Err(crate::Incomplete::Exhausted));
    }

    #[test]
    fn test_computable_identity_dyn_computable() {
        let identity: ComputableIdentity<i32> = 100.into();
        let mut dyn_computable = identity.dyn_computable();
        let result = dyn_computable.try_compute().unwrap();
        assert_eq!(result, 100);
    }
}
