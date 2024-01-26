use super::Check;
use crate::Result;

/// Test implementation of a check
pub struct TestCheck;

impl TestCheck {
    pub fn new() -> Self {
        TestCheck
    }
}

impl Check for TestCheck {
    /// Always pass
    fn check(&mut self) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {}
