use crate::{Completable, DynComputable, Incomplete};
use cancel_this::Cancellable;
use serde::{Deserialize, Serialize};

/// A generic trait implemented by types that represent a "computation".
///
/// To advance the computation, repeatedly call [`Computable::try_compute`] until a value is
/// returned. Once the value is returned, the computable becomes "exhausted" and will return
/// [`Incomplete::Exhausted`].
///
/// See also [`ComputableResult`] and [`crate::Computation`].
pub trait Computable<T> {
    /// Try to advance this computation, returning a value once the computation is done.
    fn try_compute(&mut self) -> Completable<T>;

    /// Advance this computation until it either completes, is canceled, or becomes exhausted,
    /// skipping over all suspended states.
    ///
    /// This method is identical to repeatedly calling [`Computable::try_compute`] until it
    /// returns something other than [`Incomplete::Suspended`].
    ///
    /// Note that this method can loop forever if the computation never completes and keeps
    /// returning [`Incomplete::Suspended`].
    fn compute_completable(&mut self) -> Completable<T> {
        loop {
            match self.try_compute() {
                Ok(value) => return Ok(value),
                Err(Incomplete::Suspended) => continue,
                Err(e) => return Err(e),
            }
        }
    }

    /// Advance this computation until completion, skipping over all suspended states.
    ///
    /// # Panics
    ///
    /// Panics if called on an exhausted computation, i.e., if [`Computable::try_compute`] returns
    /// [`Incomplete::Exhausted`]. If you want to handle exhaustion gracefully, use
    /// [`Computable::compute_completable`] instead.
    fn compute(&mut self) -> Cancellable<T> {
        match self.compute_completable() {
            Ok(value) => Ok(value),
            Err(Incomplete::Suspended) => unreachable!(
                "`compute_completable` never returns `Incomplete::Suspended` by definition."
            ),
            Err(Incomplete::Cancelled(c)) => Err(c),
            Err(Incomplete::Exhausted) => panic!("Called `compute` on an exhausted `Computable`."),
        }
    }

    /// Utility method to convert this [`Computable`] to a dynamic type.
    fn dyn_computable(self) -> DynComputable<T>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

/// A result-like object that stores the result of a [`Computable`] for later use.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ComputableResult<T, C: Computable<T>> {
    computable: C,
    result: Option<T>,
}

impl<T, C: Computable<T>> From<C> for ComputableResult<T, C> {
    fn from(value: C) -> Self {
        ComputableResult {
            computable: value,
            result: None,
        }
    }
}

impl<T, C: Computable<T>> ComputableResult<T, C> {
    /// Create a new [`ComputableResult`] from an instance of [`Computable`].
    pub fn new(computable: C) -> Self {
        computable.into()
    }

    /// Advance the inner [`Computable`] and return its result or return a reference
    /// to the already computed result.
    pub fn try_compute(&mut self) -> Completable<&T> {
        if self.result.is_none() {
            let result = self.computable.try_compute()?;
            self.result = Some(result);
        }

        if let Some(result) = self.result.as_ref() {
            return Ok(result);
        }

        unreachable!("Both `result` and `computable` cannot be `None`.")
    }

    /// A reference to the computed result, assuming it is already available.
    pub fn result_ref(&self) -> Option<&T> {
        self.result.as_ref()
    }

    /// The computed result, assuming it is already available.
    pub fn result(self) -> Option<T> {
        self.result
    }

    /// A reference to the underlying computation, assuming it is still available.
    pub fn computable_ref(&self) -> &C {
        &self.computable
    }

    /// The underlying computation, assuming it is still available.
    pub fn computable(self) -> C {
        self.computable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComputableIdentity, Incomplete};

    #[test]
    fn test_computable_result_from() {
        let identity: ComputableIdentity<i32> = 42.into();
        let result: ComputableResult<i32, ComputableIdentity<i32>> = identity.into();
        assert!(result.result_ref().is_none());
    }

    #[test]
    fn test_computable_result_new() {
        let identity: ComputableIdentity<i32> = 100.into();
        let mut result = ComputableResult::new(identity);
        assert!(result.result_ref().is_none());

        let computed = result.try_compute().unwrap();
        assert_eq!(*computed, 100);
        assert_eq!(result.result_ref(), Some(&100));
    }

    #[test]
    fn test_computable_result_try_compute_multiple_times() {
        let identity: ComputableIdentity<String> = "test".to_string().into();
        let mut result = ComputableResult::new(identity);

        let first = result.try_compute().unwrap();
        assert_eq!(*first, "test");
        let first_ptr = first as *const String;

        // The second call should return the same reference
        let second = result.try_compute().unwrap();
        assert_eq!(*second, "test");
        let second_ptr = second as *const String;
        assert_eq!(first_ptr, second_ptr);
    }

    #[test]
    fn test_computable_result_result() {
        let identity: ComputableIdentity<i32> = 42.into();
        let mut result = ComputableResult::new(identity);
        let _ = result.try_compute().unwrap();

        let value = result.result();
        assert_eq!(value, Some(42));
    }

    #[test]
    fn test_computable_result_result_none() {
        let identity: ComputableIdentity<i32> = 42.into();
        let result = ComputableResult::new(identity);
        let value = result.result();
        assert_eq!(value, None);
    }

    #[test]
    fn test_computable_result_computable_ref() {
        let identity: ComputableIdentity<i32> = 42.into();
        let result = ComputableResult::new(identity);
        let _computable_ref = result.computable_ref();
    }

    #[test]
    fn test_computable_result_computable() {
        let identity: ComputableIdentity<i32> = 42.into();
        let result = ComputableResult::new(identity);
        let mut computable = result.computable();
        let value = computable.try_compute().unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_dyn_computable() {
        let identity: ComputableIdentity<i32> = 42.into();
        let mut dyn_computable = identity.dyn_computable();
        let result = dyn_computable.try_compute().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_compute_method() {
        let mut identity: ComputableIdentity<i32> = 42.into();
        let result = identity.compute().unwrap();
        assert_eq!(result, 42);
    }

    // Test with a computable that suspends
    struct SuspendingComputable {
        count: u32,
        target: u32,
    }

    impl Computable<u32> for SuspendingComputable {
        fn try_compute(&mut self) -> Completable<u32> {
            self.count += 1;
            if self.count < self.target {
                Err(Incomplete::Suspended)
            } else {
                Ok(self.count)
            }
        }
    }

    #[test]
    fn test_compute_with_suspensions() {
        let mut computable = SuspendingComputable {
            count: 0,
            target: 3,
        };
        let result = computable.compute().unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn test_try_compute_with_suspensions() {
        let mut computable = SuspendingComputable {
            count: 0,
            target: 3,
        };

        // The first call should suspend
        assert_eq!(computable.try_compute(), Err(Incomplete::Suspended));
        // The second call should suspend
        assert_eq!(computable.try_compute(), Err(Incomplete::Suspended));
        // The third call should complete
        assert_eq!(computable.try_compute(), Ok(3));
    }
}
