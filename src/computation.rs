use crate::{Algorithm, Completable, Computable, Stateful};
use cancel_this::is_cancelled;
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
        is_cancelled!()?;
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

    fn state_mut(&mut self) -> &mut STATE {
        &mut self.state
    }
}

impl<CONTEXT, STATE, OUTPUT, STEP: ComputationStep<CONTEXT, STATE, OUTPUT>>
    Algorithm<CONTEXT, STATE, OUTPUT> for Computation<CONTEXT, STATE, OUTPUT, STEP>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Algorithm, Computable, Incomplete, Stateful};

    struct SimpleStep;

    impl ComputationStep<i32, u32, String> for SimpleStep {
        fn step(context: &i32, state: &mut u32) -> Completable<String> {
            *state += 1;
            if *state < 3 {
                Err(Incomplete::Suspended)
            } else {
                Ok(format!("context={}, state={}", context, state))
            }
        }
    }

    #[test]
    fn test_computation_from_parts() {
        let computation = Computation::<i32, u32, String, SimpleStep>::from_parts(42, 0);
        assert_eq!(*computation.context(), 42);
        assert_eq!(*computation.state(), 0);
    }

    #[test]
    fn test_computation_into_parts() {
        let computation = Computation::<i32, u32, String, SimpleStep>::from_parts(100, 5);
        let (context, state) = computation.into_parts();
        assert_eq!(context, 100);
        assert_eq!(state, 5);
    }

    #[test]
    fn test_computation_state_mut() {
        let mut computation = Computation::<i32, u32, String, SimpleStep>::from_parts(42, 0);
        *computation.state_mut() = 10;
        assert_eq!(*computation.state(), 10);
    }

    #[test]
    fn test_computation_try_compute() {
        let mut computation = Computation::<i32, u32, String, SimpleStep>::from_parts(42, 0);

        // The first call should suspend
        assert_eq!(computation.try_compute(), Err(Incomplete::Suspended));
        assert_eq!(*computation.state(), 1);

        // The second call should suspend
        assert_eq!(computation.try_compute(), Err(Incomplete::Suspended));
        assert_eq!(*computation.state(), 2);

        // The third call should complete
        let result = computation.try_compute().unwrap();
        assert_eq!(result, "context=42, state=3");
        assert_eq!(*computation.state(), 3);
    }

    #[test]
    fn test_computation_compute() {
        let mut computation = Computation::<i32, u32, String, SimpleStep>::from_parts(100, 0);
        let result = computation.compute().unwrap();
        assert_eq!(result, "context=100, state=3");
        assert_eq!(*computation.state(), 3);
    }

    #[test]
    fn test_computation_configure() {
        let computation = Computation::<i32, u32, String, SimpleStep>::configure(50, 0u32);
        assert_eq!(*computation.context(), 50);
        assert_eq!(*computation.state(), 0);
    }

    #[test]
    fn test_computation_run() {
        let result = Computation::<i32, u32, String, SimpleStep>::run(200, 0u32).unwrap();
        assert_eq!(result, "context=200, state=3");
    }

    #[test]
    fn test_computation_dyn_algorithm() {
        let computation = Computation::<i32, u32, String, SimpleStep>::from_parts(42, 0);
        let mut dyn_algorithm = computation.dyn_algorithm();
        let result = dyn_algorithm.compute().unwrap();
        assert_eq!(result, "context=42, state=3");
    }

    struct ImmediateStep;

    impl ComputationStep<(), (), i32> for ImmediateStep {
        fn step(_context: &(), _state: &mut ()) -> Completable<i32> {
            Ok(42)
        }
    }

    #[test]
    fn test_computation_immediate_completion() {
        let mut computation = Computation::<(), (), i32, ImmediateStep>::from_parts((), ());
        let result = computation.try_compute().unwrap();
        assert_eq!(result, 42);
    }

    struct NeverCompleteStep;

    impl ComputationStep<(), (), i32> for NeverCompleteStep {
        fn step(_context: &(), _state: &mut ()) -> Completable<i32> {
            Err(Incomplete::Suspended)
        }
    }

    #[test]
    fn test_computation_never_completes() {
        let mut computation = Computation::<(), (), i32, NeverCompleteStep>::from_parts((), ());
        // This will loop forever in compute(), so we test try_compute instead
        assert_eq!(computation.try_compute(), Err(Incomplete::Suspended));
        assert_eq!(computation.try_compute(), Err(Incomplete::Suspended));
    }
}
