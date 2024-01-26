use super::Trigger;
use crate::Result;
use std::sync::mpsc::Sender;

/// A trigger that runs the checks periodically
pub struct ScheduleTrigger;

impl Trigger for ScheduleTrigger {
    fn listen(&self, tx: &Sender<Option<()>>) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
