use super::{Trigger, TriggerError};
use log::info;
use std::{collections::HashMap, sync::mpsc::Sender};

const TRIGGER_NAME: &str = "ONCE";

/// A trigger that runs the checks once and then exits.
pub struct OnceTrigger;

impl Trigger for OnceTrigger {
    /// Starts a trigger that runs once and terminates after.
    fn listen(&self, tx: Sender<Option<HashMap<String, String>>>) -> Result<(), TriggerError> {
        info!("Triggering only once.");
        let context: HashMap<String, String> =
            HashMap::from([("TRIGGER_NAME".to_string(), TRIGGER_NAME.to_string())]);
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
        let (tx, rx) = mpsc::channel::<Option<HashMap<String, String>>>();

        trigger.listen(tx).unwrap();

        let msgs: Vec<_> = rx.iter().collect();
        assert_eq!(
            vec![
                Some(HashMap::from([(
                    "TRIGGER_NAME".to_string(),
                    TRIGGER_NAME.to_string()
                )])),
                None
            ],
            msgs
        );
    }
}
