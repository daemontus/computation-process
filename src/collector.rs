use crate::{Completable, Computable, DynGeneratable, Incomplete};

pub struct Collector<ITEM, COLLECTION: Default + Extend<ITEM>> {
    generator: DynGeneratable<ITEM>,
    collector: Option<COLLECTION>,
}

impl<ITEM, COLLECTION: Default + Extend<ITEM>> From<DynGeneratable<ITEM>>
    for Collector<ITEM, COLLECTION>
{
    fn from(value: DynGeneratable<ITEM>) -> Self {
        Collector {
            generator: value,
            collector: Some(Default::default()),
        }
    }
}

impl<ITEM, COLLECTION: Default + Extend<ITEM>> Computable<COLLECTION>
    for Collector<ITEM, COLLECTION>
{
    fn try_compute(&mut self) -> Completable<COLLECTION> {
        match self.generator.try_next() {
            None => Ok(self
                .collector
                .take()
                .expect("Trying to poll a stale computable.")),
            Some(Ok(item)) => {
                let collector = self
                    .collector
                    .as_mut()
                    .expect("Trying to poll a stale computable.");
                collector.extend(std::iter::once(item));
                Err(Incomplete::Suspended)
            }
            Some(Err(Incomplete::Suspended)) => Err(Incomplete::Suspended),
            Some(Err(Incomplete::Cancelled(c))) => Err(Incomplete::Cancelled(c)),
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
    #[should_panic(expected = "stale")]
    fn test_collector_stale_after_completion() {
        let generator = TestGenerator {
            items: vec![1],
            index: 0,
        };
        let mut collector: Collector<i32, Vec<i32>> = generator.dyn_generatable().into();

        // First call adds item and suspends
        assert_eq!(collector.try_compute(), Err(Incomplete::Suspended));

        // The second call completes
        let _ = collector.try_compute().unwrap();

        // Third call should panic (stale)
        let _ = collector.try_compute();
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
