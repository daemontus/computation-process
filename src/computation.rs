use crate::{Algorithm, Completable, Computable, Stateful};
use std::marker::PhantomData;

pub trait ComputationStep<CONTEXT, STATE, OUTPUT> {
    fn step(context: &CONTEXT, state: &mut STATE) -> Completable<OUTPUT>;
}

pub struct Computation<CONTEXT, STATE, OUTPUT, STEP: ComputationStep<CONTEXT, STATE, OUTPUT>> {
    context: CONTEXT,
    state: STATE,
    _phantom: PhantomData<(OUTPUT, STEP)>,
}

impl<CONTEXT, STATE, OUTPUT, STEP: ComputationStep<CONTEXT, STATE, OUTPUT>> Computable<OUTPUT>
    for Computation<CONTEXT, STATE, OUTPUT, STEP>
{
    fn try_compute(&mut self) -> Completable<OUTPUT> {
        STEP::step(&self.context, &mut self.state)
    }
}

impl<CONTEXT, STATE, OUTPUT, STEP: ComputationStep<CONTEXT, STATE, OUTPUT>> Stateful<CONTEXT, STATE>
    for Computation<CONTEXT, STATE, OUTPUT, STEP>
{
    fn from_parts(context: CONTEXT, state: STATE) -> Self
    where
        Self: Sized + 'static,
    {
        Computation {
            context,
            state,
            _phantom: Default::default(),
        }
    }

    fn into_parts(self) -> (CONTEXT, STATE) {
        (self.context, self.state)
    }

    fn context(&self) -> &CONTEXT {
        &self.context
    }

    fn state(&self) -> &STATE {
        &self.state
    }
}

impl<CONTEXT, STATE, OUTPUT, STEP: ComputationStep<CONTEXT, STATE, OUTPUT>>
    Algorithm<CONTEXT, STATE, OUTPUT> for Computation<CONTEXT, STATE, OUTPUT, STEP>
{
}
