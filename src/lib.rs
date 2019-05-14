//! A minimal set of abstractions to create application with the Elm architecture in Rust.
//!
//! Differences to the Elm architecture:
//! - States are mutable, we trust Rust.
//! - No predefined HTML view model, any component may support multiple view models.

mod application;
pub use application::*;

mod cmd;
pub use cmd::*;

mod component;
pub use component::*;

mod executor;
pub use executor::*;
use std::thread;

/// A simple exector that uses std::thread::spawn.
pub struct ThreadSpawnExecutor {}

impl Executor for ThreadSpawnExecutor {
    fn spawn(&mut self, f: Box<dyn Fn() -> () + 'static + Send>) {
        let _jh = thread::spawn(move || f());
    }
}