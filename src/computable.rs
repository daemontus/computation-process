use crate::{Completable, DynComputable, Incomplete};
use cancel_this::Cancellable;
use serde::{Deserialize, Serialize};

/// A generic trait implemented by types that represent a "computation".
///
/// To advance the computation, repeatedly call [`Computable::try_compute`] until a value is
/// returned. Once the value is returned, the computable becomes "stale" and is allowed to panic.
///
/// See also [`ComputableResult`] and [`crate::Computation`].
pub trait Computable<T> {
    /// Try to advance this computation, returning a value once the computation is done.
    fn try_compute(&mut self) -> Completable<T>;

    /// Advance this computation until completion, skipping over all suspended states.
    fn compute(&mut self) -> Cancellable<T> {
        loop {
            match self.try_compute() {
                Ok(value) => return Ok(value),
                Err(Incomplete::Cancelled(c)) => return Err(c),
                Err(Incomplete::Suspended) => continue,
            }
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
