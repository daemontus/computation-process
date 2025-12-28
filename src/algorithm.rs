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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Computation, ComputationStep, Generator, GeneratorStep, Incomplete};

    struct TestComputationStep;

    impl ComputationStep<i32, u32, String> for TestComputationStep {
        fn step(context: &i32, state: &mut u32) -> crate::Completable<String> {
            *state += 1;
            if *state < 2 {
                Err(Incomplete::Suspended)
            } else {
                Ok(format!("done-{}", context))
            }
        }
    }

    #[test]
    fn test_stateful_from_parts() {
        let stateful = Computation::<i32, u32, String, TestComputationStep>::from_parts(42, 0);
        assert_eq!(*stateful.context(), 42);
        assert_eq!(*stateful.state(), 0);
    }

    #[test]
    fn test_stateful_configure() {
        let stateful = Computation::<i32, u32, String, TestComputationStep>::configure(100, 5u32);
        assert_eq!(*stateful.context(), 100);
        assert_eq!(*stateful.state(), 5);
    }

    #[test]
    fn test_stateful_into_parts() {
        let stateful = Computation::<i32, u32, String, TestComputationStep>::from_parts(50, 10);
        let (context, state) = stateful.into_parts();
        assert_eq!(context, 50);
        assert_eq!(state, 10);
    }

    #[test]
    fn test_stateful_context() {
        let stateful = Computation::<i32, u32, String, TestComputationStep>::from_parts(200, 0);
        assert_eq!(*stateful.context(), 200);
    }

    #[test]
    fn test_stateful_state() {
        let stateful = Computation::<i32, u32, String, TestComputationStep>::from_parts(0, 42);
        assert_eq!(*stateful.state(), 42);
    }

    #[test]
    fn test_stateful_state_mut() {
        let mut stateful = Computation::<i32, u32, String, TestComputationStep>::from_parts(0, 0);
        *stateful.state_mut() = 100;
        assert_eq!(*stateful.state(), 100);
    }

    #[test]
    fn test_algorithm_run() {
        let result = Computation::<i32, u32, String, TestComputationStep>::run(42, 0u32).unwrap();
        assert_eq!(result, "done-42");
    }

    #[test]
    fn test_algorithm_dyn_algorithm() {
        let algorithm = Computation::<i32, u32, String, TestComputationStep>::from_parts(100, 0);
        let mut dyn_algorithm = algorithm.dyn_algorithm();
        let result = dyn_algorithm.compute().unwrap();
        assert_eq!(result, "done-100");
    }

    struct TestGeneratorStep;

    impl GeneratorStep<i32, u32, String> for TestGeneratorStep {
        fn step(context: &i32, state: &mut u32) -> crate::Completable<Option<String>> {
            *state += 1;
            if *state <= 2 {
                Ok(Some(format!("{}-{}", context, state)))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_gen_algorithm_computation() {
        let generator = Generator::<i32, u32, String, TestGeneratorStep>::from_parts(42, 0);
        let mut computation = generator.computation::<Vec<String>>();
        let result = computation.compute().unwrap();
        assert_eq!(result, vec!["42-1", "42-2"]);
    }

    #[test]
    fn test_gen_algorithm_computation_hashset() {
        let generator = Generator::<i32, u32, String, TestGeneratorStep>::from_parts(42, 0);
        let mut computation = generator.computation::<std::collections::HashSet<String>>();
        let result = computation.compute().unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains("42-1"));
        assert!(result.contains("42-2"));
    }

    #[test]
    fn test_gen_algorithm_dyn_algorithm() {
        let generator = Generator::<i32, u32, String, TestGeneratorStep>::from_parts(100, 0);
        let mut dyn_algorithm = generator.dyn_algorithm();
        let item = dyn_algorithm.try_next().unwrap().unwrap();
        assert_eq!(item, "100-1");
    }

    #[test]
    fn test_stateful_configure_with_conversions() {
        // Test that configure works with Into conversions
        let stateful =
            Computation::<i32, u32, String, TestComputationStep>::configure(42i16, 10u16);
        assert_eq!(*stateful.context(), 42i32);
        assert_eq!(*stateful.state(), 10u32);
    }
}
