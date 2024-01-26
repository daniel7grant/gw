use super::Action;
use crate::Result;

/// Test implementation of an action
pub struct TestAction;

impl TestAction {
    pub fn new() -> Self {
        Self
    }
}

impl Action for TestAction {
    /// Do nothing
    fn run(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
