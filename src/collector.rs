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
