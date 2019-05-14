use crate::{Component, Executor};
use std::sync::{Arc, Mutex};

/// Application
/// TODO: do we need to be generic over S here?
/// TODO: we use N here because the notification
///       function can not be boxed, because we need to clone it.
/// TODO: Can we abstract this generic monster away, afer all it's all private e.g.
///       hide / box behind a trait that is only generic over E.
pub struct Application<S, E, N>
where
    S: Component<S, E>,
    E: Send,
{
    state: Box<Component<S, E>>,
    executor: Box<Executor>,
    notify: N,
    pending: Arc<Mutex<Vec<E>>>,
}

impl<S, E, N> Application<S, E, N>
where
    S: Component<S, E>,
    E: 'static + Send,
    N: Fn() -> () + 'static + Send + Clone,
{
    /// Creates an application from a component and an executor,
    /// and a asynchronous callback the informs when update should be called.

    pub fn new(
        root: impl Component<S, E> + 'static,
        executor: impl Executor + 'static,
        notify: N,
    ) -> Application<S, E, N> {
        Application {
            state: Box::new(root),
            executor: Box::new(executor),
            notify,
            pending: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Schedule an event. Note that this does not call update(),
    /// neither invoke the notification notify callback,
    /// the client has to take care of that.
    pub fn schedule(&mut self, event: E) -> &mut Self {
        self.pending.lock().unwrap().push(event);
        self
    }

    /// Notify the external callback that onr or more new
    /// eventa are pending. This directly calls the notification callback
    /// and does nothing else, expecting that the client calls update() in
    /// turn.
    pub fn notify(&self) -> &Self {
        (self.notify)();
        self
    }

    /// Update the application's state. This delivers _all_ pending events and
    /// schedules the commands to the executor.
    pub fn update(&mut self) -> &mut Self {
        for e in self.pending.lock().unwrap().drain(..) {
            let cmd = self.state.update(e);
            for f in cmd.unpack() {
                let notify = self.notify.clone();
                let pending = self.pending.clone();
                let async_fn = move || {
                    let r = f();
                    pending.lock().unwrap().push(r);
                    notify();
                };
                self.executor.spawn(Box::new(async_fn));
            }
        }
        self
    }
}
