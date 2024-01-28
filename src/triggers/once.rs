use super::Trigger;
use crate::Result;
use std::sync::mpsc::Sender;

/// A trigger that runs the checks once and then exits.
pub struct OnceTrigger;

impl Trigger for OnceTrigger {
    /// Starts a trigger that runs once and terminates after.
    fn listen(&self, tx: Sender<Option<()>>) -> Result<()> {
        tx.send(Some(()))?;
        tx.send(None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn it_should_trigger_once_and_stop() {
        let trigger = OnceTrigger;
        let (tx, rx) = mpsc::channel::<Option<()>>();

        trigger.listen(tx).unwrap();

        let msgs: Vec<_> = rx.iter().collect();
        assert_eq!(vec![Some(()), None], msgs);
    }
}
