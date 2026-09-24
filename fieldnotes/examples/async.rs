use std::future::Future;
use std::sync::mpsc::{self, Sender};
use std::task::{Context, Poll, Waker};
use std::thread::{self, JoinHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Key { run: u64, request: u64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Settings { cost: u32, capacity: u32 }
impl Settings {
    fn valid(self) -> bool { self.cost > 0 && self.cost <= self.capacity && self.capacity <= 100 }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportError { Malformed, InvalidSettings }
#[derive(Debug)]
struct Completion { key: Key, result: Result<Settings, ImportError> }
#[derive(Clone, Copy)]
struct AcceptedInput { key: Key, settings: Settings }
#[derive(Clone, Copy)]
struct Request { key: Key, completed: bool }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct World { run: u64, tick: u64, settings: Settings }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reception { Staged, Failed, Ignored }
struct App {
    world: World,
    next_request: u64,
    current: Option<Request>,
    staged: Option<AcceptedInput>,
    error: Option<ImportError>,
}
impl App {
    fn new() -> Self {
        Self { world: World { run: 1, tick: 0, settings: Settings { cost: 2, capacity: 10 } },
            next_request: 0, current: None, staged: None, error: None }
    }
    fn request(&mut self) -> Key {
        self.next_request = self.next_request.checked_add(1).expect("request identity exhausted");
        let key = Key { run: self.world.run, request: self.next_request };
        self.current = Some(Request { key, completed: false });
        self.staged = None;
        self.error = None;
        key
    }
    fn receive(&mut self, completion: Completion) -> Reception {
        let Some(request) = self.current.as_mut() else { return Reception::Ignored; };
        if completion.key.run != self.world.run || completion.key != request.key || request.completed {
            return Reception::Ignored;
        }
        request.completed = true;
        let result = completion.result.and_then(|settings| {
            if settings.valid() { Ok(settings) } else { Err(ImportError::InvalidSettings) }
        });
        match result {
            Ok(settings) => {
                self.staged = Some(AcceptedInput { key: completion.key, settings });
                self.error = None;
                Reception::Staged
            }
            Err(error) => {
                self.staged = None;
                self.error = Some(error);
                Reception::Failed
            }
        }
    }
    fn tick(&mut self) {
        let next_tick = self.world.tick.checked_add(1).expect("tick exhausted");
        if let Some(input) = self.staged.take() {
            if input.key.run == self.world.run && self.current.map(|request| request.key) == Some(input.key) {
                self.world.settings = input.settings;
                self.current = None;
            }
        }
        self.world.tick = next_tick;
    }
    fn cancel(&mut self) { self.current = None; self.staged = None; self.error = None; }
    fn reset(&mut self) {
        let run = self.world.run.checked_add(1).expect("run identity exhausted");
        self.cancel();
        self.next_request = 0;
        self.world = World { run, tick: 0, settings: Settings { cost: 2, capacity: 10 } };
    }
}
async fn parse_ready(payload: String) -> Result<Settings, ImportError> {
    let (cost, capacity) = payload.split_once(',').ok_or(ImportError::Malformed)?;
    let settings = Settings { cost: cost.parse().map_err(|_| ImportError::Malformed)?,
        capacity: capacity.parse().map_err(|_| ImportError::Malformed)? };
    if settings.valid() { Ok(settings) } else { Err(ImportError::InvalidSettings) }
}
// One poll of a known-ready lab future. This is not an I/O executor.
fn poll_ready<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("this probe accepts only immediately ready futures"),
    }
}
fn worker(key: Key, payload: String, output: Sender<Completion>) -> (Sender<()>, JoinHandle<()>) {
    let (start, gate) = mpsc::channel();
    let handle = thread::spawn(move || {
        gate.recv().unwrap();
        let result = poll_ready(parse_ready(payload));
        output.send(Completion { key, result }).unwrap();
    });
    (start, handle)
}
fn newer_first(old: Key, new: Key) -> [Completion; 2] {
    let (send, receive) = mpsc::channel();
    let (old_gate, old_handle) = worker(old, "3,30".to_owned(), send.clone());
    let (new_gate, new_handle) = worker(new, "4,40".to_owned(), send);
    new_gate.send(()).unwrap();
    let newer = receive.recv().unwrap();
    old_gate.send(()).unwrap();
    let older = receive.recv().unwrap();
    new_handle.join().unwrap(); old_handle.join().unwrap();
    [newer, older]
}
fn completion(key: Key, cost: u32, capacity: u32) -> Completion {
    Completion { key, result: Ok(Settings { cost, capacity }) }
}
fn main() {
    let mut app = App::new();
    let old = app.request(); let new = app.request();
    let [newer, older] = newer_first(old, new);
    println!("received new={:?} old={:?}", app.receive(newer), app.receive(older));
    println!("before tick={} cost={} capacity={}", app.world.tick, app.world.settings.cost, app.world.settings.capacity);
    app.tick();
    println!("after tick={} cost={} capacity={}", app.world.tick, app.world.settings.cost, app.world.settings.capacity);
    app.reset();
    println!("after reset stale={:?} run={}", app.receive(completion(new, 4, 40)), app.world.run);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn controlled_threads_complete_newer_first_and_only_newer_is_committed() {
        let mut app = App::new();
        let old = app.request(); let new = app.request();
        let [newer, older] = newer_first(old, new);
        assert_eq!((newer.key, older.key), (new, old));
        assert_eq!(app.receive(newer), Reception::Staged);
        assert_eq!(app.receive(older), Reception::Ignored);
        app.tick();
        assert_eq!(app.world.settings, Settings { cost: 4, capacity: 40 });
    }
    #[test]
    fn arrival_does_not_change_authoritative_state_until_an_executed_tick() {
        let mut app = App::new(); let before = app.world;
        let key = app.request();
        assert_eq!(app.receive(completion(key, 3, 30)), Reception::Staged);
        assert_eq!(app.world, before);
        app.tick();
        assert_eq!((app.world.tick, app.world.settings), (1, Settings { cost: 3, capacity: 30 }));
    }
    #[test]
    fn replacing_a_request_also_discards_its_already_staged_input() {
        let mut app = App::new(); let old = app.request();
        app.receive(completion(old, 3, 30));
        let new = app.request(); app.tick();
        assert_eq!(app.world.settings, Settings { cost: 2, capacity: 10 });
        app.receive(completion(new, 4, 40)); app.tick();
        assert_eq!(app.world.settings, Settings { cost: 4, capacity: 40 });
    }
    #[test]
    fn cancellation_ignores_late_completion_and_discards_staged_work() {
        let mut app = App::new(); let key = app.request();
        app.receive(completion(key, 3, 30)); app.cancel();
        assert_eq!(app.receive(completion(key, 4, 40)), Reception::Ignored);
        app.tick();
        assert_eq!(app.world.settings, Settings { cost: 2, capacity: 10 });
    }
    #[test]
    fn reset_rejects_old_run_even_when_request_number_is_reused() {
        let mut app = App::new(); let old = app.request();
        app.reset(); let new = app.request();
        assert_eq!(old.request, new.request); assert_ne!(old.run, new.run);
        assert_eq!(app.receive(completion(old, 3, 30)), Reception::Ignored);
        assert_eq!(app.receive(completion(new, 4, 40)), Reception::Staged);
        app.tick();
        assert_eq!((app.world.run, app.world.settings), (2, Settings { cost: 4, capacity: 40 }));
    }
    #[test]
    fn malformed_and_invalid_results_never_partially_commit_settings() {
        let mut app = App::new(); let before = app.world.settings;
        for payload in ["3,nope", "3,101", "20,10"] {
            let key = app.request();
            let result = poll_ready(parse_ready(payload.to_owned()));
            assert!(result.is_err());
            assert_eq!(app.receive(Completion { key, result }), Reception::Failed);
            app.tick(); assert_eq!(app.world.settings, before);
        }
        let key = app.request();
        assert_eq!(app.receive(completion(key, 0, 10)), Reception::Failed);
        app.tick(); assert_eq!(app.world.settings, before);
    }
    #[test]
    fn duplicate_completion_cannot_replace_the_first_accepted_result() {
        let mut app = App::new(); let key = app.request();
        assert_eq!(app.receive(completion(key, 3, 30)), Reception::Staged);
        assert_eq!(app.receive(completion(key, 4, 40)), Reception::Ignored);
        app.tick();
        assert_eq!(app.receive(completion(key, 4, 40)), Reception::Ignored);
        assert_eq!(app.world.settings, Settings { cost: 3, capacity: 30 });
    }
    #[test]
    fn stale_error_does_not_erase_a_newer_success_or_display_an_error() {
        let mut app = App::new(); let old = app.request(); let new = app.request();
        app.receive(completion(new, 4, 40));
        assert_eq!(app.receive(Completion { key: old, result: Err(ImportError::Malformed) }), Reception::Ignored);
        assert_eq!(app.error, None);
        app.tick(); assert_eq!(app.world.settings, Settings { cost: 4, capacity: 40 });
    }
    #[test]
    fn async_body_waits_for_poll_and_owned_messages_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Completion>();
        let called = std::cell::Cell::new(false);
        let future = async { called.set(true); 7 };
        assert!(!called.get());
        assert_eq!(poll_ready(future), 7);
        assert!(called.get());
        assert_eq!(poll_ready(parse_ready("3,30".to_owned())), Ok(Settings { cost: 3, capacity: 30 }));
    }
}
