use super::Trigger;
use crate::Result;
use std::sync::mpsc::Sender;

/// A trigger that runs the checks once and then exits
pub struct OnceTrigger;

impl Trigger for OnceTrigger {
    fn listen(&self, tx: &Sender<Option<()>>) -> Result<()> {
        tx.send(Some(()))?;
		tx.send(None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
