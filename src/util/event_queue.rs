use std::collections::VecDeque;

pub struct EventQueue<T> {
    events: VecDeque<T>,
}

impl<T> EventQueue<T> {
    pub fn poll(&mut self) -> Option<T> {
        self.events.pop_front()
    }

    pub fn new(&mut self, event: T) {
        self.events.push_back(event);
    }
}

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }
}
