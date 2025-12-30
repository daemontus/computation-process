use crate::{
    Collector, Completable, Computable, ComputableResult, Computation, ComputationStep, Generator,
    GeneratorStep, Incomplete, Stateful,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestContext(i32);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestState(i32);

struct TestComputationStep;

impl ComputationStep<TestContext, TestState, i32> for TestComputationStep {
    fn step(context: &TestContext, state: &mut TestState) -> Completable<i32> {
        state.0 += 1;
        if state.0 < context.0 {
            Err(Incomplete::Suspended)
        } else {
            Ok(state.0)
        }
    }
}

#[test]
fn test_computation_serialization() {
    let computation = Computation::<TestContext, TestState, i32, TestComputationStep>::from_parts(
        TestContext(10),
        TestState(5),
    );

    let serialized = serde_json::to_string(&computation).unwrap();
    let deserialized: Computation<TestContext, TestState, i32, TestComputationStep> =
        serde_json::from_str(&serialized).unwrap();

    assert_eq!(computation.context(), deserialized.context());
    assert_eq!(computation.state(), deserialized.state());
}

#[test]
fn test_computable_result_serialization() {
    let computation = Computation::<TestContext, TestState, i32, TestComputationStep>::from_parts(
        TestContext(10),
        TestState(5),
    );
    let result = ComputableResult::new(computation);

    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: ComputableResult<
        i32,
        Computation<TestContext, TestState, i32, TestComputationStep>,
    > = serde_json::from_str(&serialized).unwrap();

    assert_eq!(
        result.computable_ref().context(),
        deserialized.computable_ref().context()
    );
    assert_eq!(
        result.computable_ref().state(),
        deserialized.computable_ref().state()
    );
    assert_eq!(result.result_ref(), deserialized.result_ref());
}

struct TestGeneratorStep;

impl GeneratorStep<TestContext, TestState, i32> for TestGeneratorStep {
    fn step(context: &TestContext, state: &mut TestState) -> Completable<Option<i32>> {
        state.0 += 1;
        if state.0 < context.0 {
            Ok(Some(state.0))
        } else {
            Ok(None)
        }
    }
}

#[test]
fn test_generator_serialization() {
    let generator = Generator::<TestContext, TestState, i32, TestGeneratorStep>::from_parts(
        TestContext(10),
        TestState(5),
    );

    let serialized = serde_json::to_string(&generator).unwrap();
    let deserialized: Generator<TestContext, TestState, i32, TestGeneratorStep> =
        serde_json::from_str(&serialized).unwrap();

    assert_eq!(generator.context(), deserialized.context());
    assert_eq!(generator.state(), deserialized.state());
}

#[test]
fn test_collector_serialization() {
    let generator = Generator::<TestContext, TestState, i32, TestGeneratorStep>::from_parts(
        TestContext(10),
        TestState(5),
    );

    // Explicitly use the Generator type for Collector
    let collector =
        Collector::<i32, Vec<i32>, Generator<TestContext, TestState, i32, TestGeneratorStep>>::new(
            generator,
        );

    let serialized = serde_json::to_string(&collector).unwrap();
    let deserialized: Collector<
        i32,
        Vec<i32>,
        Generator<TestContext, TestState, i32, TestGeneratorStep>,
    > = serde_json::from_str(&serialized).unwrap();

    // Check internal state somehow?
    // We can't access fields directly, but we can run it.
    let mut deserialized = deserialized;
    let result = deserialized.compute().unwrap();
    assert_eq!(result, vec![6, 7, 8, 9]);
}
