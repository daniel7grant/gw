use super::{Trigger, TriggerError};
use crate::context::Context;
use duration_string::DurationString;
use log::info;
use std::{
    collections::HashMap,
    sync::mpsc::Sender,
    thread::sleep,
    time::{Duration, Instant},
};
use thiserror::Error;

const TRIGGER_NAME: &str = "SCHEDULE";

/// A trigger that runs the checks periodically.
///
/// This is running in an infinite loop, triggering every time.
pub struct ScheduleTrigger {
    duration: Duration,
    timeout: Option<Duration>,
}

/// Custom error describing the error cases for the ScheduleTrigger.
#[derive(Debug, Error)]
pub enum ScheduleError {
    /// Cannot send trigger with Sender. This usually because the receiver is dropped.
    #[error("cannot trigger changes, receiver hang up")]
    ReceiverHangup(#[from] std::sync::mpsc::SendError<Option<Context>>),
}

impl From<ScheduleError> for TriggerError {
    fn from(val: ScheduleError) -> Self {
        match val {
            ScheduleError::ReceiverHangup(s) => TriggerError::ReceiverHangup(s),
        }
    }
}

impl ScheduleTrigger {
    /// Creates a new ScheduleTrigger with duration.
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            timeout: None,
        }
    }

    /// Creates a new ScheduleTrigger with duration and timeout.
    pub fn new_with_timeout(duration: Duration, timeout: Duration) -> Self {
        Self {
            duration,
            timeout: Some(timeout),
        }
    }

    /// Runs one step in the scheduled time process. Returns true, if it should continue,
    /// returns false in case of an error or a timeout. One step should take exactly the duration.
    /// In case of an error it terminates or if it will reach the final timeout it will
    /// wait until the end of the timeout and returns with false.
    pub fn step(
        &self,
        tx: Sender<Option<Context>>,
        final_timeout: Option<Instant>,
    ) -> Result<bool, ScheduleError> {
        let next_check = Instant::now() + self.duration;

        let context: Context = HashMap::from([
            ("TRIGGER_NAME", TRIGGER_NAME.to_string()),
            (
                "SCHEDULE_DELAY",
                DurationString::from(self.duration).to_string(),
            ),
        ]);
        tx.send(Some(context))?;

        if let Some(final_timeout) = final_timeout {
            if next_check > final_timeout {
                let until_final_timeout = final_timeout - Instant::now();
                sleep(until_final_timeout);
                return Ok(false);
            }
        }
        // TODO: handle overlaps
        let until_next_check = next_check - Instant::now();
        sleep(until_next_check);

        // We should handle if the sleep was too long and it went over the timeout
        if let Some(final_timeout) = final_timeout {
            Ok(Instant::now() < final_timeout)
        } else {
            Ok(true)
        }
    }
}

impl Trigger for ScheduleTrigger {
    /// Starts a scheduled trigger on a new thread, starting the steps in a loop.
    /// Every step triggers and then waits the given duration. In case of an error,
    /// it terminates or if it will reach the final timeout it will wait until
    /// the end of the timeout and return.
    fn listen(&self, tx: Sender<Option<Context>>) -> Result<(), TriggerError> {
        let final_timeout = self.timeout.map(|t| Instant::now() + t);
        info!(
            "Starting schedule in every {}.",
            DurationString::new(self.duration)
        );

        loop {
            let should_continue = self.step(tx.clone(), final_timeout)?;
            if !should_continue {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triggers::TriggerError;
    use std::{sync::mpsc, time::Instant};

    #[test]
    fn it_should_be_created_from_duration() {
        let trigger = ScheduleTrigger::new(Duration::from_millis(100));
        assert_eq!(Duration::from_millis(100), trigger.duration);
        assert_eq!(None, trigger.timeout);
    }

    #[test]
    fn it_should_be_created_from_duration_and_timeout() {
        let trigger = ScheduleTrigger::new_with_timeout(
            Duration::from_millis(100),
            Duration::from_millis(200),
        );
        assert_eq!(Duration::from_millis(100), trigger.duration);
        assert_eq!(Some(Duration::from_millis(200)), trigger.timeout);
    }

    #[test]
    fn it_should_trigger_every_100_ms() -> Result<(), TriggerError> {
        let trigger = ScheduleTrigger::new(Duration::from_millis(100));
        let (tx, rx) = mpsc::channel::<Option<Context>>();

        for _ in 0..5 {
            let start = Instant::now();

            let should_continue = trigger.step(tx.clone(), None)?;
            assert!(should_continue);

            // It should be close to the timings
            let msg = rx.recv().unwrap();
            let diff = start.elapsed();
            assert!(
                diff >= Duration::from_millis(95),
                "Diff {} should be later than 95ms.",
                DurationString::from(diff)
            );

            // It should contain the hashmap
            let context = msg.unwrap();
            assert_eq!(TRIGGER_NAME, context.get("TRIGGER_NAME").unwrap());
            assert_eq!("100ms", context.get("SCHEDULE_DELAY").unwrap());
        }

        Ok(())
    }

    #[test]
    fn it_should_not_continue_after_the_timeout() -> Result<(), TriggerError> {
        let trigger = ScheduleTrigger::new(Duration::from_millis(100));
        let (tx, _rx) = mpsc::channel::<Option<Context>>();

        let start = Instant::now();
        let final_timeout = start + Duration::from_millis(350);
        for _ in 0..5 {
            let should_continue = trigger.step(tx.clone(), Some(final_timeout))?;

            // First three should pass, last two fail
            if Instant::now() < final_timeout {
                assert!(
                    should_continue,
                    "Should continue after {} passed, before 300ms.",
                    DurationString::from(start.elapsed())
                );
            } else {
                assert!(
                    !should_continue,
                    "Should continue after {} passed, after 300ms.",
                    DurationString::from(start.elapsed())
                );
            };
        }

        Ok(())
    }

    #[test]
    fn it_should_not_trigger_on_a_send_error() {
        let trigger = ScheduleTrigger::new(Duration::from_millis(100));
        let (tx, rx) = mpsc::channel::<Option<Context>>();

        // Close receiving end, to create a send error
        drop(rx);

        let final_timeout = Instant::now() + Duration::from_millis(350);
        let result = trigger.step(tx.clone(), Some(final_timeout));

        // It should fail, because of ReceiverHangup
        assert!(
            matches!(result, Err(ScheduleError::ReceiverHangup(_)),),
            "{result:?} should be ReceiverHangup"
        );
    }
}
