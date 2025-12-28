use crate::{Completable, DynGeneratable};
use cancel_this::Cancellable;

/// An alternative to [`crate::Computable`] which is intended for generators.
///
/// The computation is finished once [`Generatable::try_next`] returns `None`.
pub trait Generatable<T>: Iterator<Item = Cancellable<T>> {
    fn try_next(&mut self) -> Option<Completable<T>>;

    /// Utility method to convert this [`Generatable`] to a dynamic type.
    fn dyn_generatable(self) -> DynGeneratable<T>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}
