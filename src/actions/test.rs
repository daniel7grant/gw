use std::cell::RefCell;

use super::Action;
use crate::Result;

/// Test implementation of an action
pub struct TestAction {
    pub calls: RefCell<u8>,
}

impl TestAction {
    pub fn new() -> Self {
        Self {
            calls: RefCell::new(0),
        }
    }
    pub fn get_calls(&self) -> u8 {
        self.calls.take()
    }
}

impl Action for TestAction {
    /// Do nothing
    fn run(&self) -> Result<()> {
        let mut c = self.calls.borrow_mut();
        *c += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
