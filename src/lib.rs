//! # Computation process
//!
//! A Rust library for defining stateful computations (and generators) that support
//! suspend/resume, interleaving, cancellation, and serialization.
//!
//! This crate does not use `unsafe`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//!
//! ## Overview
//!
//! This library provides abstractions for defining "long-running" computations.
//! The concepts are similar to asynchronous code but offer features that were never
//! a priority in async programming. **The target audience are projects that implement
//! CPU-intensive, long-running computations but require granular control over the
//! computation state.**
//!
//! ## Key Features
//!
//! - **Cancellation:** Each computation can be forcefully stopped using cooperative
//!   cancellation (compatible with the [`cancel-this`](https://crates.io/crates/cancel-this) crate).
//! - **Suspend/resume:** A computation can define safe suspend points. During these points,
//!   it is safe to serialize, interleave, or otherwise "transfer" the computation.
//! - **Interleaving:** Suspend points allow safely interleaving multiple computations on a
//!   single thread with priority-based scheduling.
//! - **Serialization:** The state of each computation is isolated and can be saved/restored.
//!
//! ## Core Concepts
//!
//! - [`Completable<T>`]: A result type that can be incomplete (`Suspended`, `Cancelled`, or `Exhausted`).
//! - [`Computable<T>`]: A trait for objects that can be driven to completion by calling [`Computable::try_compute`].
//! - [`Algorithm<CTX, STATE, T>`]: Extends [`Computable`] with access to context and state.
//! - [`Generatable<T>`]: Like [`Computable`], but produces a stream of values.
//! - [`GenAlgorithm<CTX, STATE, T>`]: Extends [`Generatable`] with context and state.
//! - [`Computation`] and [`Generator`]: Default implementations using step functions.
//!
//! ## Quick Example
//!
//! ```rust
//! use computation_process::{Computation, ComputationStep, Completable, Incomplete, Computable, Stateful};
//!
//! struct CountingStep;
//!
//! impl ComputationStep<u32, u32, u32> for CountingStep {
//!     fn step(target: &u32, count: &mut u32) -> Completable<u32> {
//!         *count += 1;
//!         if *count >= *target {
//!             Ok(*count)
//!         } else {
//!             Err(Incomplete::Suspended)
//!         }
//!     }
//! }
//!
//! let mut computation = Computation::<u32, u32, u32, CountingStep>::from_parts(5, 0);
//! let result = computation.compute().unwrap();
//! assert_eq!(result, 5);
//! ```

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

pub use algorithm::{Algorithm, GenAlgorithm, Stateful};
pub use collector::Collector;
pub use completable::{Completable, Incomplete};
pub use computable::{Computable, ComputableResult};
pub use computable_identity::ComputableIdentity;
pub use computation::{Computation, ComputationStep};
pub use generatable::Generatable;
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{Computation, ComputationStep, Generator, GeneratorStep, Incomplete};

    struct SumComputationStep;

    impl ComputationStep<Vec<i32>, i32, i32> for SumComputationStep {
        fn step(context: &Vec<i32>, state: &mut i32) -> Completable<i32> {
            if *state < context.len() as i32 {
                *state += 1;
                Err(Incomplete::Suspended)
            } else {
                Ok(context.iter().sum())
            }
        }
    }

    #[test]
    fn test_dyn_computable_integration() {
        let identity: ComputableIdentity<i32> = 42.into();
        let mut dyn_computable: DynComputable<i32> = identity.dyn_computable();
        let result = dyn_computable.try_compute().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_dyn_algorithm_integration() {
        let computation = Computation::<Vec<i32>, i32, i32, SumComputationStep>::from_parts(
            vec![1, 2, 3, 4, 5],
            0,
        );
        let mut dyn_algorithm: DynAlgorithm<Vec<i32>, i32, i32> = computation.dyn_algorithm();
        let result = dyn_algorithm.compute().unwrap();
        assert_eq!(result, 15);
    }

    #[test]
    fn test_dyn_algorithm_as_computable() {
        let computation =
            Computation::<Vec<i32>, i32, i32, SumComputationStep>::from_parts(vec![10, 20], 0);
        let mut dyn_algorithm: DynAlgorithm<Vec<i32>, i32, i32> = computation.dyn_algorithm();
        // Test that DynAlgorithm implements Computable
        let result = dyn_algorithm.try_compute();
        assert!(matches!(result, Err(Incomplete::Suspended)));

        let result = dyn_algorithm.compute().unwrap();
        assert_eq!(result, 30);
    }

    struct RangeGeneratorStep;

    impl GeneratorStep<i32, i32, i32> for RangeGeneratorStep {
        fn step(context: &i32, state: &mut i32) -> Completable<Option<i32>> {
            *state += 1;
            if *state <= *context {
                Ok(Some(*state))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_dyn_generatable_integration() {
        let generator = Generator::<i32, i32, i32, RangeGeneratorStep>::from_parts(5, 0);
        let mut dyn_generatable: DynGeneratable<i32> = generator.dyn_generatable();

        let mut items = Vec::new();
        while let Some(item) = dyn_generatable.try_next() {
            items.push(item.unwrap());
        }

        assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_dyn_gen_algorithm_integration() {
        let generator = Generator::<i32, i32, i32, RangeGeneratorStep>::from_parts(3, 0);
        let mut dyn_gen_algorithm: DynGenAlgorithm<i32, i32, i32> = generator.dyn_algorithm();

        let mut items = Vec::new();
        while let Some(item) = dyn_gen_algorithm.try_next() {
            items.push(item.unwrap());
        }

        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn test_dyn_gen_algorithm_as_generatable() {
        let generator = Generator::<i32, i32, i32, RangeGeneratorStep>::from_parts(3, 0);
        let mut dyn_gen_algorithm: DynGenAlgorithm<i32, i32, i32> = generator.dyn_algorithm();

        // Test that DynGenAlgorithm implements Generatable
        let item = dyn_gen_algorithm.try_next().unwrap().unwrap();
        assert_eq!(item, 1);
    }

    #[test]
    fn test_end_to_end_computation_with_suspensions() {
        let computation =
            Computation::<Vec<i32>, i32, i32, SumComputationStep>::from_parts(vec![1, 2, 3], 0);
        let mut dyn_computable: DynComputable<i32> = computation.dyn_computable();

        // The first call should suspend
        assert!(matches!(
            dyn_computable.try_compute(),
            Err(Incomplete::Suspended)
        ));

        // The second call should suspend
        assert!(matches!(
            dyn_computable.try_compute(),
            Err(Incomplete::Suspended)
        ));

        // The third call should suspend
        assert!(matches!(
            dyn_computable.try_compute(),
            Err(Incomplete::Suspended)
        ));

        // Fourth call should complete
        let result = dyn_computable.try_compute().unwrap();
        assert_eq!(result, 6);
    }

    #[test]
    fn test_end_to_end_generator_collection() {
        let generator = Generator::<i32, i32, i32, RangeGeneratorStep>::from_parts(4, 0);
        let mut computation = generator.computation::<Vec<i32>>();
        let result = computation.compute().unwrap();
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_end_to_end_generator_with_dyn_types() {
        let generator = Generator::<i32, i32, i32, RangeGeneratorStep>::from_parts(3, 0);
        let dyn_generatable: DynGeneratable<i32> = generator.dyn_generatable();
        let mut collector: Collector<i32, Vec<i32>> = dyn_generatable.into();
        let result = collector.compute().unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_algorithm_run_static_method() {
        let result =
            Computation::<Vec<i32>, i32, i32, SumComputationStep>::run(vec![5, 10, 15], 0i32)
                .unwrap();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_computable_result_integration() {
        let computation =
            Computation::<Vec<i32>, i32, i32, SumComputationStep>::from_parts(vec![1, 2], 0);
        let mut result = ComputableResult::new(computation);

        // The computation suspends multiple times, so we need to call try_compute until it succeeds
        let computed = loop {
            match result.try_compute() {
                Ok(value) => break value,
                Err(Incomplete::Suspended) => continue,
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        };
        assert_eq!(*computed, 3);
        let computed_ptr = computed as *const i32;

        // Second call returns cached result
        let cached = result.try_compute().unwrap();
        assert_eq!(*cached, 3);
        let cached_ptr = cached as *const i32;
        assert_eq!(computed_ptr, cached_ptr);
    }

    #[test]
    fn test_multiple_dyn_computable() {
        let identity1: ComputableIdentity<i32> = 10.into();
        let identity2: ComputableIdentity<i32> = 20.into();

        let mut dyn1: DynComputable<i32> = identity1.dyn_computable();
        let mut dyn2: DynComputable<i32> = identity2.dyn_computable();

        assert_eq!(dyn1.try_compute().unwrap(), 10);
        assert_eq!(dyn2.try_compute().unwrap(), 20);
    }

    // Cancellation integration tests

    struct LongRunningStep;

    impl ComputationStep<u32, u32, u32> for LongRunningStep {
        fn step(target: &u32, state: &mut u32) -> Completable<u32> {
            *state += 1;
            if *state >= *target {
                Ok(*state)
            } else {
                Err(Incomplete::Suspended)
            }
        }
    }

    #[test]
    fn test_computation_cancellation() {
        use cancel_this::{CancelAtomic, on_trigger};

        let trigger = CancelAtomic::new();
        trigger.cancel(); // Pre-cancel

        let mut computation = Computation::<u32, u32, u32, LongRunningStep>::from_parts(1000, 0);

        let result = on_trigger(trigger, || computation.try_compute());

        assert!(matches!(result, Err(Incomplete::Cancelled(_))));
    }

    #[test]
    fn test_computation_compute_with_cancellation() {
        use cancel_this::{CancelAtomic, on_trigger};

        let trigger = CancelAtomic::new();
        trigger.cancel(); // Pre-cancel

        let mut computation = Computation::<u32, u32, u32, LongRunningStep>::from_parts(1000, 0);

        let result = on_trigger(trigger, || computation.compute());

        assert!(result.is_err());
    }

    struct CancellableGeneratorStep;

    impl GeneratorStep<u32, u32, u32> for CancellableGeneratorStep {
        fn step(max: &u32, state: &mut u32) -> Completable<Option<u32>> {
            *state += 1;
            if *state <= *max {
                Ok(Some(*state))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_generator_iterator_cancellation() {
        use cancel_this::{CancelAtomic, on_trigger};

        let trigger = CancelAtomic::new();
        trigger.cancel(); // Pre-cancel

        let mut generator =
            Generator::<u32, u32, u32, CancellableGeneratorStep>::from_parts(100, 0);

        // on_trigger expects Result, so we wrap the iterator call
        let result = on_trigger(trigger, || match generator.next() {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        });

        // Should be canceled
        assert!(result.is_err());
    }

    #[test]
    fn test_collector_with_cancellation() {
        use cancel_this::{CancelAtomic, on_trigger};

        let trigger = CancelAtomic::new();
        trigger.cancel(); // Pre-cancel

        let generator = Generator::<u32, u32, u32, CancellableGeneratorStep>::from_parts(100, 0);
        let mut collector: Collector<u32, Vec<u32>> = generator.dyn_generatable().into();

        let result = on_trigger(trigger, || collector.compute());

        assert!(result.is_err());
    }
}
