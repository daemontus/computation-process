use crate::{Completable, DynGeneratable};
use cancel_this::Cancellable;

/// An alternative to [`crate::Computable`] which is intended for generators.
///
/// The computation is finished once [`Generatable::try_next`] returns `None`.
pub trait Generatable<T>: Iterator<Item = Cancellable<T>> {
    /// Try to advance the generator and return the next item.
    ///
    /// Returns:
    /// - `Some(Ok(item))` when an item is available
    /// - `Some(Err(Incomplete::Suspended))` when the generator needs to yield control
    /// - `Some(Err(Incomplete::Cancelled(_)))` when the computation was canceled
    /// - `None` when the generator is exhausted
    fn try_next(&mut self) -> Option<Completable<T>>;

    /// Utility method to convert this [`Generatable`] to a dynamic type.
    fn dyn_generatable(self) -> DynGeneratable<T>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}
