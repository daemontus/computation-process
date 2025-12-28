use crate::generatable::Generatable;
use crate::{Completable, GenAlgorithm, Incomplete, Stateful};
use cancel_this::Cancellable;
use std::marker::PhantomData;

pub trait GeneratorStep<CONTEXT, STATE, ITEM> {
    fn step(context: &CONTEXT, state: &mut STATE) -> Completable<Option<ITEM>>;
}

pub struct Generator<CONTEXT, STATE, ITEM, STEP: GeneratorStep<CONTEXT, STATE, ITEM>> {
    context: CONTEXT,
    state: STATE,
    _phantom: PhantomData<(ITEM, STEP)>,
}

impl<CONTEXT, STATE, ITEM, STEP: GeneratorStep<CONTEXT, STATE, ITEM>> Iterator
    for Generator<CONTEXT, STATE, ITEM, STEP>
{
    type Item = Cancellable<ITEM>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match STEP::step(&self.context, &mut self.state) {
                Ok(None) => return None,
                Ok(Some(item)) => return Some(Ok(item)),
                Err(Incomplete::Suspended) => continue,
                Err(Incomplete::Cancelled(c)) => return Some(Err(c)),
            }
        }
    }
}

impl<CONTEXT, STATE, OUTPUT, STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>> Generatable<OUTPUT>
    for Generator<CONTEXT, STATE, OUTPUT, STEP>
{
    fn try_next(&mut self) -> Option<Completable<OUTPUT>> {
        STEP::step(&self.context, &mut self.state).transpose()
    }
}

impl<CONTEXT, STATE, ITEM, STEP: GeneratorStep<CONTEXT, STATE, ITEM>> Stateful<CONTEXT, STATE>
    for Generator<CONTEXT, STATE, ITEM, STEP>
{
    fn from_parts(context: CONTEXT, state: STATE) -> Self
    where
        Self: Sized + 'static,
    {
        Generator {
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

impl<CONTEXT, STATE, ITEM, STEP: GeneratorStep<CONTEXT, STATE, ITEM>>
    GenAlgorithm<CONTEXT, STATE, ITEM> for Generator<CONTEXT, STATE, ITEM, STEP>
{
}
