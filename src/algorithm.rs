use crate::generatable::Generatable;
use crate::{Collector, Computable, DynAlgorithm, DynGenAlgorithm};
use cancel_this::Cancellable;

/// A shared interface of objects that provide access to
/// an immutable `CONTEXT` and mutable `STATE`.
pub trait Stateful<CONTEXT, STATE> {
    /// Create new [`Stateful`] instance using values that can be
    /// converted to `CONTEXT` and `STATE`.
    fn configure<I1: Into<CONTEXT>, I2: Into<STATE>>(context: I1, initial_state: I2) -> Self
    where
        Self: Sized + 'static,
    {
        Self::from_parts(context.into(), initial_state.into())
    }

    /// Create new [`Stateful`] instance from `CONTEXT` and `STATE`.
    fn from_parts(context: CONTEXT, state: STATE) -> Self
    where
        Self: Sized + 'static;

    /// Destruct the [`Stateful`] instance into `CONTEXT` and `STATE` objects.
    fn into_parts(self) -> (CONTEXT, STATE);

    /// Access to the underlying immutable `CONTEXT`.
    fn context(&self) -> &CONTEXT;

    /// Access to the underlying `STATE`.
    fn state(&self) -> &STATE;

    /// Access to the underlying `STATE` as a mutable reference.
    ///
    /// Keep in mind that having a consistent state is important for the correctness
    /// of [`Algorithm`] and [`GenAlgorithm`]. You should modify the internal state
    /// of a [`Stateful`] object only in rare, well-defined situations.
    fn state_mut(&mut self) -> &mut STATE;
}

/// Extends [`Computable`] trait with immutable `CONTEXT` and mutable `STATE`.
pub trait Algorithm<CONTEXT, STATE, OUTPUT>: Computable<OUTPUT> + Stateful<CONTEXT, STATE> {
    /// Configure and immediately execute the computation, skipping over all suspended states.
    fn run<I1: Into<CONTEXT>, I2: Into<STATE>>(
        context: I1,
        initial_state: I2,
    ) -> Cancellable<OUTPUT>
    where
        Self: Sized + 'static,
    {
        Self::from_parts(context.into(), initial_state.into()).compute()
    }

    /// Convert to a dynamic [`Algorithm`] variant.
    fn dyn_algorithm(self) -> DynAlgorithm<CONTEXT, STATE, OUTPUT>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

/// Extends [`Generatable`] trait with immutable `CONTEXT` and mutable `STATE`.
pub trait GenAlgorithm<CONTEXT, STATE, OUTPUT>:
    Generatable<OUTPUT> + Stateful<CONTEXT, STATE>
{
    /// Convert a [`GenAlgorithm`] into a [`Computable`] object that collects all values
    /// into a `COLLECTION`.
    fn computation<COLLECTION: Default + Extend<OUTPUT> + 'static>(
        self,
    ) -> impl Computable<COLLECTION>
    where
        Self: Sized + 'static,
    {
        Collector::<OUTPUT, COLLECTION>::from(self.dyn_generatable())
    }

    /// Convert to a dynamic [`GenAlgorithm`] variant.
    fn dyn_algorithm(self) -> DynGenAlgorithm<CONTEXT, STATE, OUTPUT>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}
