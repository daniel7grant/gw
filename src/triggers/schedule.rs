use super::Trigger;
use crate::Result;

/// A trigger that runs the checks periodically
struct ScheduleTrigger;

impl Trigger for ScheduleTrigger {
    fn listen(self: Self) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
