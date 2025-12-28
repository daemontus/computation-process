use crate::{Completable, Computable};

/// Implementation of [`Computable`] that simply returns the provided value.
pub struct ComputableIdentity<T> {
    value: Option<T>,
}

impl<T> From<T> for ComputableIdentity<T> {
    fn from(value: T) -> Self {
        ComputableIdentity { value: Some(value) }
    }
}

impl<T> Computable<T> for ComputableIdentity<T> {
    fn try_compute(&mut self) -> Completable<T> {
        if let Some(result) = self.value.take() {
            Ok(result)
        } else {
            panic!("Called `try_compute` on a stale `Computable`.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Computable;

    #[test]
    fn test_computable_identity_from() {
        let identity: ComputableIdentity<i32> = 42.into();
        let mut identity = identity;
        let result = identity.try_compute().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_computable_identity_compute() {
        let mut identity: ComputableIdentity<String> = "hello".to_string().into();
        let result = identity.compute().unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    #[should_panic(expected = "stale")]
    fn test_computable_identity_stale() {
        let mut identity: ComputableIdentity<i32> = 42.into();
        let _ = identity.try_compute().unwrap();
        // The second call should panic
        let _ = identity.try_compute();
    }

    #[test]
    fn test_computable_identity_dyn_computable() {
        let identity: ComputableIdentity<i32> = 100.into();
        let mut dyn_computable = identity.dyn_computable();
        let result = dyn_computable.try_compute().unwrap();
        assert_eq!(result, 100);
    }
}
