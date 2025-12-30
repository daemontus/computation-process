use crate::{Completable, Computable, DynGeneratable, Generatable, Incomplete};
use std::marker::PhantomData;

/// A [`Computable`] that collects all items from a [`Generatable`] into a collection.
///
/// This is useful for converting a generator/stream of items into a single collected result.
/// The collection type must implement [`Default`] and [`Extend`].
///
/// # Example
///
/// ```rust
/// use computation_process::{Generator, GeneratorStep, Completable, Computable, Collector, Stateful, Generatable};
///
/// struct RangeStep;
///
/// impl GeneratorStep<u32, u32, u32> for RangeStep {
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
/// let generator = Generator::<u32, u32, u32, RangeStep>::from_parts(3, 0);
/// let mut collector: Collector<u32, Vec<u32>> = generator.dyn_generatable().into();
/// let result = collector.compute().unwrap();
/// assert_eq!(result, vec![1, 2, 3]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        bound = "G: serde::Serialize + for<'a> serde::Deserialize<'a>, COLLECTION: serde::Serialize + for<'a> serde::Deserialize<'a>"
    )
)]
pub struct Collector<ITEM, COLLECTION, G = DynGeneratable<ITEM>>
where
    COLLECTION: Default + Extend<ITEM>,
    G: Generatable<ITEM>,
{
    generator: G,
    collector: Option<COLLECTION>,
    #[cfg_attr(feature = "serde", serde(skip))]
    _phantom: PhantomData<ITEM>,
}

impl<ITEM, COLLECTION, G> Collector<ITEM, COLLECTION, G>
where
    COLLECTION: Default + Extend<ITEM>,
    G: Generatable<ITEM>,
{
    /// Create a new collector for the given generator.
    pub fn new(generator: G) -> Self {
        Collector {
            generator,
            collector: Some(Default::default()),
            _phantom: Default::default(),
        }
    }
}

impl<ITEM, COLLECTION: Default + Extend<ITEM>> From<DynGeneratable<ITEM>>
    for Collector<ITEM, COLLECTION, DynGeneratable<ITEM>>
{
    fn from(value: DynGeneratable<ITEM>) -> Self {
        Collector::new(value)
    }
}

impl<ITEM, COLLECTION, G> Computable<COLLECTION> for Collector<ITEM, COLLECTION, G>
where
    COLLECTION: Default + Extend<ITEM>,
    G: Generatable<ITEM>,
{
    fn try_compute(&mut self) -> Completable<COLLECTION> {
        match self.generator.try_next() {
            None => {
                if let Some(collector) = self.collector.take() {
                    Ok(collector)
                } else {
                    Err(Incomplete::Exhausted)
                }
            }
            Some(Ok(item)) => {
                if let Some(collector) = self.collector.as_mut() {
                    collector.extend(std::iter::once(item));
                    Err(Incomplete::Suspended)
                } else {
                    Err(Incomplete::Exhausted)
                }
            }
            Some(Err(Incomplete::Suspended)) => Err(Incomplete::Suspended),
            Some(Err(Incomplete::Cancelled(c))) => Err(Incomplete::Cancelled(c)),
            Some(Err(Incomplete::Exhausted)) => Err(Incomplete::Exhausted),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Computable, Generatable, Incomplete};
    use cancel_this::Cancellable;

    struct TestGenerator {
        items: Vec<i32>,
        index: usize,
    }

    impl Iterator for TestGenerator {
        type Item = Cancellable<i32>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index < self.items.len() {
                let item = self.items[self.index];
                self.index += 1;
                Some(Ok(item))
            } else {
                None
            }
        }
    }

    impl Generatable<i32> for TestGenerator {
        fn try_next(&mut self) -> Option<Completable<i32>> {
            if self.index < self.items.len() {
                let item = self.items[self.index];
                self.index += 1;
                Some(Ok(item))
            } else {
                None
            }
        }
    }

    #[test]
    fn test_collector_from() {
        let generator = TestGenerator {
            items: vec![1, 2, 3],
            index: 0,
        };
        let collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();
        // Should have initialized with Some(Default::default())
        assert!(collector.collector.is_some());
    }

    #[test]
    fn test_collector_basic() {
        let generator = TestGenerator {
            items: vec![1, 2, 3],
            index: 0,
        };
        let mut collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();

        // First item
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // Second item
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // Third item
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // Collection complete
        let result = collector.try_compute().unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_collector_compute() {
        let generator = TestGenerator {
            items: vec![10, 20, 30],
            index: 0,
        };
        let mut collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();

        let result = collector.compute().unwrap();
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn test_collector_empty() {
        let generator = TestGenerator {
            items: vec![],
            index: 0,
        };
        let mut collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();

        let result = collector.try_compute().unwrap();
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_collector_single_item() {
        let generator = TestGenerator {
            items: vec![42],
            index: 0,
        };
        let mut collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();

        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));
        let result = collector.try_compute().unwrap();
        assert_eq!(result, vec![42]);
    }

    #[test]
    fn test_collector_with_hashset() {
        let generator = TestGenerator {
            items: vec![1, 2, 2, 3],
            index: 0,
        };
        let mut collector: Collector<i32, std::collections::HashSet<i32>> =
            generator.dyn_generatable().into();

        // Collect all items
        let result = loop {
            match collector.try_compute() {
                Ok(result) => break result,
                Err(Incomplete::Suspended) => continue,
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        };

        assert_eq!(result.len(), 3); // HashSet deduplicates
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }

    struct SuspendingGenerator {
        items: Vec<i32>,
        index: usize,
        first_call: bool,
    }

    impl Iterator for SuspendingGenerator {
        type Item = Cancellable<i32>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index < self.items.len() {
                let item = self.items[self.index];
                self.index += 1;
                Some(Ok(item))
            } else {
                None
            }
        }
    }

    impl Generatable<i32> for SuspendingGenerator {
        fn try_next(&mut self) -> Option<Completable<i32>> {
            if self.index < self.items.len() {
                // Suspend on the very first call, then return items normally
                if self.first_call {
                    self.first_call = false;
                    Some(Err(Incomplete::Suspended))
                } else {
                    let item = self.items[self.index];
                    self.index += 1;
                    Some(Ok(item))
                }
            } else {
                None
            }
        }
    }

    #[test]
    fn test_collector_with_suspensions() {
        let generator = SuspendingGenerator {
            items: vec![1, 2, 3],
            index: 0,
            first_call: true,
        };
        let mut collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();

        // First call: generator suspends on the first call
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // Second call: generator returns the first item, collector adds it and suspends
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // Third call: generator returns the second item, collector adds it and suspends
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // Fourth call: generator returns the third item, collector adds it and suspends
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // Fifth call: generator returns None, collector completes
        let result = collector.try_compute().unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_collector_exhausted_after_completion() {
        let generator = TestGenerator {
            items: vec![1],
            index: 0,
        };
        let mut collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();

        // First call adds item and suspends
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // The second call completes
        let _ = collector.try_compute().unwrap();

        // Third call should return Exhausted
        assert_eq!(collector.try_compute(), Err(Incomplete::Exhausted));
    }

    struct CancellingGenerator {
        cancelled: bool,
    }

    impl Iterator for CancellingGenerator {
        type Item = Cancellable<i32>;

        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }

    impl Generatable<i32> for CancellingGenerator {
        fn try_next(&mut self) -> Option<Completable<i32>> {
            if !self.cancelled {
                self.cancelled = true;
                Some(Err(
                    Incomplete::Cancelled(cancel_this::Cancelled::default()),
                ))
            } else {
                None
            }
        }
    }

    #[test]
    fn test_collector_cancellation() {
        let generator = CancellingGenerator { cancelled: false };
        let mut collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();

        let result = collector.try_compute();
        assert!(matches!(result, Err(Incomplete::Cancelled(_))));
    }
}
