[![Crates.io](https://img.shields.io/crates/v/computation-process?style=flat-square)](https://crates.io/crates/computation-process)
[![Api Docs](https://img.shields.io/badge/docs-api-yellowgreen?style=flat-square)](https://docs.rs/computation-process/)
[![Continuous integration](https://img.shields.io/github/actions/workflow/status/daemontus/computation-process/build.yml?branch=main&style=flat-square)](https://github.com/daemontus/computation-process/actions/workflows/build.yml)
[![Coverage](https://img.shields.io/codecov/c/github/daemontus/computation-process?style=flat-square)](https://codecov.io/gh/daemontus/computation-process)
[![GitHub issues](https://img.shields.io/github/issues/daemontus/computation-process?style=flat-square)](https://github.com/daemontus/computation-process/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/daemontus/computation-process?style=flat-square)](https://github.com/daemontus/computation-process/commits/main)
[![Crates.io](https://img.shields.io/crates/l/computation-process?style=flat-square)](https://github.com/daemontus/computation-process/blob/main/LICENSE)

# `computation-process` (Suspendable CPU-intensive tasks in Rust)

This library provides abstractions for defining "long-running" computations.
The concepts in `computation-process` are often similar to "normal" asynchronous 
code, but offer certain features that were never a priority in asynchronous 
programming. **The target audience are projects that implement CPU-intensive,
long-running computations but require granular control over the computation state
(e.g., because the computation is controlled from a user interface).**

> This is currently still very "experimental." I am releasing this on `crates.io`
> to allow some initial large-scale usage experiments, but please bear in mind that the
> API can change in the future.

Specific problems for which `computation-process` offers an opinionated design pattern:

 - **Cancellation:** Each computation can be forcefully stopped using cooperative cancellation
   (compatible with the [`cancel-this`](https://crates.io/crates/cancel-this) crate).
   A canceled computation should remain in a consistent state from which it can be 
   restarted even though some intermediate results may need to be recomputed.
 - **Suspend/resume:** A computation can define safe suspend points (based on polling).
   During these points, it is safe to serialize/interleave or otherwise "transfer" the 
   computation without losing any progress.
 - **Interleaving and priority scheduling:** Presence of suspend points allows us to 
   safely interleave multiple computations on a single thread. Interleaving can use the 
   inner state of each computation for priority-based scheduling.
 - **Serialization:** The state of each computation is isolated into a dedicated object 
   and can be therefore saved/restored during any of the suspend points.

## Overview of concepts

 - `Cancellable` and `Completable`: Function returns `Cancellable<T>` if it can be interrupted
   by a cancellation token from `cancel-this`. A function returns `Completable<T>`
   if it is cancellable, and it also returns `Incomplete::Suspended` whenever it is safe to
   suspend.
 - `Computable<T>` and `Algorithm<CTX, STATE, T>`: An object implements `Computable<T>` if
   it can be driven into completion by repeatedly calling `try_compute`, which returns 
   `Completable<T>`. An `Algorithm` is then an extension of `Computable` that can be 
   configured (i.e., created) using `CTX` and `STATE` objects, and it provides access to 
   these objects during computation.
 - Similarly, `Generatable<T>` and `GenAlgorithm<CTX, STATE, T>` are variants of `Computable`
   and `Algorithm` that do not produce a single value, but rather a "stream" of `T` values
   (like a cancellable/suspendable iterator).
 - A `Computation` and `Generator` are the default implementations of these interfaces.
   They delegate the actual computation to `ComputationStep` and `GeneratorStep` while
   taking care of the remaining "boilerplate."

## Quickstart

**It is highly recommended to enable LTO when using `computation-process` because it allows better
inlining of our computation abstractions (at the expense of build time).**

To enable LTO, add this to your `Cargo.toml`:

```
[profile.release]
lto = true
```

### A suspendable computation

```rust
use computation_process::{Completable, Computable, Computation, ComputationStep, Incomplete, Stateful};

struct CountingStep;

impl ComputationStep<u32, u32, u32> for CountingStep {
    fn step(target: &u32, count: &mut u32) -> Completable<u32> {
        *count += 1;
        if *count >= *target {
            Ok(*count)
        } else {
            Err(Incomplete::Suspended)
        }
    }
}

fn example() {
   let mut computation = Computation::<u32, u32, u32, CountingStep>::from_parts(5, 0);
   assert_eq!(computation.compute().unwrap(), 5);  
}
```

### A suspendable generator

```rust
use computation_process::{Completable, Generatable, Generator, GeneratorStep, Stateful};

struct RangeStep;

impl GeneratorStep<u32, u32, u32> for RangeStep {
    fn step(max: &u32, current: &mut u32) -> Completable<Option<u32>> {
        *current += 1;
        if *current <= *max {
            Ok(Some(*current))
        } else {
            Ok(None)
        }
    }
}

fn example() {
   let mut generator = Generator::<u32, u32, u32, RangeStep>::from_parts(3, 0);
   assert_eq!(generator.try_next(), Some(Ok(1)));
   assert_eq!(generator.try_next(), Some(Ok(2)));
   assert_eq!(generator.try_next(), Some(Ok(3)));
   assert_eq!(generator.try_next(), None);  
}
```