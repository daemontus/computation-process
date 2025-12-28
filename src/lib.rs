// All traits/structs have dedicated modules for encapsulation, and we then re-export
// these types here for easier public usage.

mod algorithm;
mod collector;
mod completable;
mod computable;
mod computable_identity;
mod computation;
mod generatable;
mod generator;

use crate::generatable::Generatable;
pub use algorithm::{Algorithm, GenAlgorithm, Stateful};
pub use collector::Collector;
pub use completable::{Completable, Incomplete};
pub use computable::{Computable, ComputableResult};
pub use computable_identity::ComputableIdentity;
pub use computation::{Computation, ComputationStep};
pub use generator::{Generator, GeneratorStep};

/// A type alias for `Box<dyn Computable<T>>`.
pub type DynComputable<T> = Box<dyn Computable<T>>;

/// A type alias for `Box<dyn Generatable<T>>`.
pub type DynGeneratable<T> = Box<dyn Generatable<T>>;

/// A type alias for `Box<dyn Algorithm<CONTEXT, STATE, OUTPUT>>`.
pub type DynAlgorithm<CONTEXT, STATE, OUTPUT> = Box<dyn Algorithm<CONTEXT, STATE, OUTPUT>>;

/// A type alias for `Box<dyn GenAlgorithm<CONTEXT, STATE, OUTPUT>>`.
pub type DynGenAlgorithm<CONTEXT, STATE, ITEM> = Box<dyn GenAlgorithm<CONTEXT, STATE, ITEM>>;

// Dummy implementations of Computable / Generatable for dynamic objects, because these
// are not implemented automatically.

impl<T> Computable<T> for DynComputable<T> {
    fn try_compute(&mut self) -> Completable<T> {
        (**self).try_compute()
    }
}

impl<CONTEXT, STATE, OUTPUT> Computable<OUTPUT> for DynAlgorithm<CONTEXT, STATE, OUTPUT> {
    fn try_compute(&mut self) -> Completable<OUTPUT> {
        (**self).try_compute()
    }
}

impl<T> Generatable<T> for DynGeneratable<T> {
    fn try_next(&mut self) -> Option<Completable<T>> {
        (**self).try_next()
    }
}

impl<CONTEXT, STATE, OUTPUT> Generatable<OUTPUT> for DynGenAlgorithm<CONTEXT, STATE, OUTPUT> {
    fn try_next(&mut self) -> Option<Completable<OUTPUT>> {
        (**self).try_next()
    }
}
