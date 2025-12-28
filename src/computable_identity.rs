use crate::{Completable, Computable};

/// Implementation of [`Computable`] that simply returns the provided value.
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
            panic!("Called `try_compute` on a stale `Computable`.");
        }
    }
}
