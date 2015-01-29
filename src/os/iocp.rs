use error::MioResult;
use os::event;
use os::IoDesc;

pub struct Selector;

impl Selector {
    pub fn new() -> MioResult<Selector> {
        panic!();
    }

    /// Wait for events from the OS
    pub fn select(&mut self, evts: &mut Events, timeout_ms: usize) -> MioResult<()> {
        panic!();
    }

    /// Register event interests for the given IO handle with the OS
    pub fn register(&mut self, io: &IoDesc, token: usize, interests: event::Interest, opts: event::PollOpt) -> MioResult<()> {
        panic!();
    }

    /// Register event interests for the given IO handle with the OS
    pub fn reregister(&mut self, io: &IoDesc, token: usize, interests: event::Interest, opts: event::PollOpt) -> MioResult<()> {
        panic!();
    }

    /// Deregister event interests for the given IO handle with the OS
    pub fn deregister(&mut self, io: &IoDesc) -> MioResult<()> {
        panic!();
    }
}


pub struct Events;

impl Events {
    pub fn new() -> Events {
         panic!();
    }

    #[inline]
    pub fn len(&self) -> usize {
         panic!();
    }

    #[inline]
    pub fn get(&self, idx: usize) -> event::IoEvent {
        panic!();
    }
}
