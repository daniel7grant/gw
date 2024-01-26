use super::Trigger;
use crate::Result;
use std::sync::mpsc::Sender;

/// A trigger that runs on an HTTP request
pub struct HttpTrigger;

impl Trigger for HttpTrigger {
    fn listen(&self, tx: &Sender<Option<()>>) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
