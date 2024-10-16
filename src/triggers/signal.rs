use super::{Trigger, TriggerError};
use crate::context::Context;
use log::debug;
use std::sync::mpsc::Sender;

const _TRIGGER_NAME: &str = "SIGNAL";

/// A trigger that terminates the program on a signal.
pub struct SignalTrigger;

impl SignalTrigger {
    #[cfg(unix)]
    fn listen_inner<I>(&self, tx: Sender<Option<Context>>, signals: I) -> Result<(), TriggerError>
    where
        I: IntoIterator<Item = i32>,
    {
        use log::error;
        for signal in signals.into_iter() {
            debug!("Got signal {signal}, terminating after all actions finished.");
            if tx.send(None).is_err() {
                error!("Failed terminating the application with signal {signal}.");
            }
        }

        Ok(())
    }
}

impl Trigger for SignalTrigger {
    /// Starts a trigger that iterates over signals and terminates the program.
    #[cfg(unix)]
    fn listen(&self, tx: Sender<Option<Context>>) -> Result<(), TriggerError> {
        use log::warn;
        use signal_hook::{
            consts::TERM_SIGNALS,
            iterator::{exfiltrator::SignalOnly, SignalsInfo},
        };
        let signals = SignalsInfo::<SignalOnly>::new(TERM_SIGNALS);
        if let Ok(mut signals) = signals {
            self.listen_inner(tx, &mut signals)?;
        } else {
            warn!("Failed setting up signal handler.");
        }

        Ok(())
    }

    #[cfg(not(unix))]
    fn listen(&self, _tx: Sender<Option<Context>>) -> Result<(), TriggerError> {
        debug!("Signal handlers are not supported on non-unix systems.");

        Ok(())
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn it_should_trigger_on_the_first_signal() {
        let trigger = SignalTrigger;
        let (tx, rx) = mpsc::channel::<Option<Context>>();

        let signals = vec![9];

        trigger.listen_inner(tx, signals).unwrap();

        let msgs: Vec<_> = rx.iter().collect();
        assert_eq!(vec![None], msgs);
    }
}
