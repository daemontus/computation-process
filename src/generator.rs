use crate::generatable::Generatable;
use crate::{Completable, GenAlgorithm, Incomplete, Stateful};
use cancel_this::{Cancellable, is_cancelled};
use std::marker::PhantomData;

/// Defines a single step of a [`Generator`].
///
/// Implement this trait to define the logic for generating items.
/// Each call to `step` should either:
/// - Return `Ok(Some(item))` to yield an item
/// - Return `Ok(None)` when the generator is exhausted
/// - Return `Err(Incomplete::Suspended)` to yield control without producing an item
/// - Return `Err(Incomplete::Cancelled(_))` if cancellation was detected
///
/// # Type Parameters
///
/// - `CONTEXT`: Immutable configuration/input for the generator
/// - `STATE`: Mutable state that persists across steps
/// - `ITEM`: The type of items produced by the generator
pub trait GeneratorStep<CONTEXT, STATE, ITEM> {
    /// Execute one step of the generator.
    ///
    /// Returns `Some(item)` to yield an item, or `None` when exhausted.
    fn step(context: &CONTEXT, state: &mut STATE) -> Completable<Option<ITEM>>;
}

/// A stateful generator that can be suspended and resumed.
///
/// `Generator` is the default implementation of [`GenAlgorithm`]. It delegates the
/// actual generation logic to a [`GeneratorStep`] implementation while handling
/// the boilerplate of state management and cancellation checking.
///
/// `Generator` implements both [`Generatable`] (for suspendable iteration) and
/// [`Iterator`] (for convenient collection, skipping suspensions automatically).
///
/// # Type Parameters
///
/// - `CONTEXT`: Immutable configuration passed to each step
/// - `STATE`: Mutable state that persists across steps
/// - `ITEM`: The type of items produced
/// - `STEP`: The [`GeneratorStep`] implementation that defines the generation logic
///
/// # Example
///
/// ```rust
/// use computation_process::{Generator, GeneratorStep, Completable, Generatable, Stateful};
///
/// struct CountStep;
///
/// impl GeneratorStep<u32, u32, u32> for CountStep {
///     fn step(max: &u32, current: &mut u32) -> Completable<Option<u32>> {
///         *current += 1;
///         if *current <= *max {
///             Ok(Some(*current))
///         } else {
///             Ok(None)
///         }
///     }
/// }
///
/// let mut generator = Generator::<u32, u32, u32, CountStep>::from_parts(3, 0);
/// assert_eq!(generator.try_next(), Some(Ok(1)));
/// assert_eq!(generator.try_next(), Some(Ok(2)));
/// assert_eq!(generator.try_next(), Some(Ok(3)));
/// assert_eq!(generator.try_next(), None);
/// ```
#[derive(Debug)]
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
            if let Err(e) = is_cancelled!() {
                return Some(Err(e));
            }

            match STEP::step(&self.context, &mut self.state) {
                Ok(None) => return None,
                Ok(Some(item)) => return Some(Ok(item)),
                Err(Incomplete::Suspended) => continue,
                Err(Incomplete::Cancelled(c)) => return Some(Err(c)),
                Err(Incomplete::Exhausted) => return None,
            }
        }
    }
}

impl<CONTEXT, STATE, OUTPUT, STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>> Generatable<OUTPUT>
    for Generator<CONTEXT, STATE, OUTPUT, STEP>
{
    fn try_next(&mut self) -> Option<Completable<OUTPUT>> {
        if let Err(e) = is_cancelled!() {
            return Some(Err(Incomplete::Cancelled(e)));
        }
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

    fn state_mut(&mut self) -> &mut STATE {
        &mut self.state
    }
}

impl<CONTEXT, STATE, ITEM, STEP: GeneratorStep<CONTEXT, STATE, ITEM>>
    GenAlgorithm<CONTEXT, STATE, ITEM> for Generator<CONTEXT, STATE, ITEM, STEP>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GenAlgorithm, Generatable, Incomplete, Stateful};
    use cancel_this::Cancellable;

    struct SimpleGeneratorStep;

    impl GeneratorStep<i32, u32, String> for SimpleGeneratorStep {
        fn step(context: &i32, state: &mut u32) -> Completable<Option<String>> {
            *state += 1;
            if *state <= 3 {
                Ok(Some(format!("item-{}-{}", context, state)))
            } else {
                Ok(None)
            }
        }
    }

    type SimpleTestGenerator = Generator<i32, u32, String, SimpleGeneratorStep>;

    #[test]
    fn test_generator_from_parts() {
        let generator = SimpleTestGenerator::from_parts(42, 0);
        assert_eq!(*generator.context(), 42);
        assert_eq!(*generator.state(), 0);
    }

    #[test]
    fn test_generator_into_parts() {
        let generator = SimpleTestGenerator::from_parts(100, 5);
        let (context, state) = generator.into_parts();
        assert_eq!(context, 100);
        assert_eq!(state, 5);
    }

    #[test]
    fn test_generator_state_mut() {
        let mut generator = SimpleTestGenerator::from_parts(42, 0);
        *generator.state_mut() = 10;
        assert_eq!(*generator.state(), 10);
    }

    #[test]
    fn test_generator_try_next() {
        let mut generator = SimpleTestGenerator::from_parts(42, 0);

        let item1 = generator.try_next().unwrap().unwrap();
        assert_eq!(item1, "item-42-1");
        assert_eq!(*generator.state(), 1);

        let item2 = generator.try_next().unwrap().unwrap();
        assert_eq!(item2, "item-42-2");
        assert_eq!(*generator.state(), 2);

        let item3 = generator.try_next().unwrap().unwrap();
        assert_eq!(item3, "item-42-3");
        assert_eq!(*generator.state(), 3);

        // After 3 items, should return None
        assert_eq!(generator.try_next(), None);
    }

    #[test]
    fn test_generator_iterator() {
        let generator = SimpleTestGenerator::from_parts(42, 0);

        let items: Vec<Cancellable<String>> = generator.collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Ok("item-42-1".to_string()));
        assert_eq!(items[1], Ok("item-42-2".to_string()));
        assert_eq!(items[2], Ok("item-42-3".to_string()));
    }

    #[test]
    fn test_generator_dyn_generatable() {
        let generator = SimpleTestGenerator::from_parts(42, 0);
        let mut dyn_gen = generator.dyn_generatable();
        let item = dyn_gen.try_next().unwrap().unwrap();
        assert_eq!(item, "item-42-1");
    }

    #[test]
    fn test_generator_dyn_algorithm() {
        let generator = SimpleTestGenerator::from_parts(42, 0);
        let mut dyn_algorithm = generator.dyn_algorithm();
        let item = dyn_algorithm.try_next().unwrap().unwrap();
        assert_eq!(item, "item-42-1");
    }

    struct SuspendingGeneratorStep;

    impl GeneratorStep<(), u32, i32> for SuspendingGeneratorStep {
        fn step(_context: &(), state: &mut u32) -> Completable<Option<i32>> {
            *state += 1;
            if *state <= 2 {
                Err(Incomplete::Suspended)
            } else if *state <= 4 {
                Ok(Some(*state as i32))
            } else {
                Ok(None)
            }
        }
    }

    type SuspendingTestGenerator = Generator<(), u32, i32, SuspendingGeneratorStep>;

    #[test]
    fn test_generator_with_suspensions() {
        let mut generator = SuspendingTestGenerator::from_parts((), 0);

        // The first call should suspend (state becomes 1)
        assert_eq!(generator.try_next(), Some(Err(Incomplete::Suspended)));
        assert_eq!(*generator.state(), 1);

        // The second call should suspend (state becomes 2)
        assert_eq!(generator.try_next(), Some(Err(Incomplete::Suspended)));
        assert_eq!(*generator.state(), 2);

        // Third call should return item (state becomes 3)
        let item = generator.try_next().unwrap().unwrap();
        assert_eq!(item, 3);
        assert_eq!(*generator.state(), 3);

        // Fourth call should return item (state becomes 4)
        let item = generator.try_next().unwrap().unwrap();
        assert_eq!(item, 4);

        // The fifth call should return None
        assert_eq!(generator.try_next(), None);
    }

    #[test]
    fn test_generator_iterator_with_suspensions() {
        let generator = SuspendingTestGenerator::from_parts((), 0);

        // Iterator should skip suspensions automatically
        let items: Vec<Cancellable<i32>> = generator.collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Ok(3));
        assert_eq!(items[1], Ok(4));
    }

    struct EmptyGeneratorStep;

    impl GeneratorStep<(), (), i32> for EmptyGeneratorStep {
        fn step(_context: &(), _state: &mut ()) -> Completable<Option<i32>> {
            Ok(None)
        }
    }

    #[test]
    fn test_empty_generator() {
        let mut generator = Generator::<(), (), i32, EmptyGeneratorStep>::from_parts((), ());
        assert_eq!(generator.try_next(), None);

        let items: Vec<Cancellable<i32>> = generator.collect();
        assert_eq!(items.len(), 0);
    }

    struct SingleItemGeneratorStep;

    impl GeneratorStep<(), (), i32> for SingleItemGeneratorStep {
        fn step(_context: &(), _state: &mut ()) -> Completable<Option<i32>> {
            Ok(Some(42))
        }
    }

    #[test]
    fn test_single_item_generator() {
        let mut generator = Generator::<(), (), i32, SingleItemGeneratorStep>::from_parts((), ());

        // This will generate infinite items since state never changes
        // But we can test that it generates at least one
        let item = generator.try_next().unwrap().unwrap();
        assert_eq!(item, 42);
    }
}
