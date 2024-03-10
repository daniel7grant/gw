use super::{Trigger, TriggerError};
use crate::context::Context;
use log::debug;
use std::{collections::HashMap, sync::mpsc::Sender};

const TRIGGER_NAME: &str = "ONCE";

/// A trigger that runs the checks once and then exits.
pub struct OnceTrigger;

impl Trigger for OnceTrigger {
    /// Starts a trigger that runs once and terminates after.
    fn listen(&self, tx: Sender<Option<Context>>) -> Result<(), TriggerError> {
        debug!("Triggering only once.");
        let context: Context = HashMap::from([("TRIGGER_NAME", TRIGGER_NAME.to_string())]);
        tx.send(Some(context))?;
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
        let (tx, rx) = mpsc::channel::<Option<Context>>();

        trigger.listen(tx).unwrap();

        let msgs: Vec<_> = rx.iter().collect();
        assert_eq!(
            vec![
                Some(HashMap::from([("TRIGGER_NAME", TRIGGER_NAME.to_string())])),
                None
            ],
            msgs
        );
    }
}
