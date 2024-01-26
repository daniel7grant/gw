use super::Trigger;
use crate::Result;
use std::sync::mpsc::Sender;

/// Test implementation for trigger
pub struct TestTrigger;

impl TestTrigger {
    pub fn new() -> Self {
        TestTrigger
    }
}

impl Trigger for TestTrigger {
    /// Trigger once and exit
    fn listen(&self, tx: &Sender<Option<()>>) -> Result<()> {
        tx.send(Some(()))?;
        tx.send(None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
