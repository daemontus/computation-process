use crate::generatable::Generatable;
use crate::{Collector, Computable, DynAlgorithm, DynGenAlgorithm};
use cancel_this::Cancellable;

pub trait Stateful<CONTEXT, STATE> {
    fn configure<I1: Into<CONTEXT>, I2: Into<STATE>>(context: I1, initial_state: I2) -> Self
    where
        Self: Sized + 'static,
    {
        Self::from_parts(context.into(), initial_state.into())
    }

    fn from_parts(context: CONTEXT, state: STATE) -> Self
    where
        Self: Sized + 'static;

    fn into_parts(self) -> (CONTEXT, STATE);

    fn context(&self) -> &CONTEXT;
    fn state(&self) -> &STATE;
}

/// Extends [`Computable`] trait with immutable `CONTEXT` and mutable `STATE`.
pub trait Algorithm<CONTEXT, STATE, OUTPUT>: Computable<OUTPUT> + Stateful<CONTEXT, STATE> {
    fn run<I1: Into<CONTEXT>, I2: Into<STATE>>(
        context: I1,
        initial_state: I2,
    ) -> Cancellable<OUTPUT>
    where
        Self: Sized + 'static,
    {
        Self::from_parts(context.into(), initial_state.into()).compute()
    }

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
    fn compute<COLLECTION: Default + Extend<OUTPUT> + 'static>(self) -> impl Computable<COLLECTION>
    where
        Self: Sized + 'static,
    {
        Collector::<OUTPUT, COLLECTION>::from(self.dyn_generatable())
    }

    fn dyn_algorithm(self) -> DynGenAlgorithm<CONTEXT, STATE, OUTPUT>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}
