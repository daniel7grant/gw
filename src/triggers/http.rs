use super::Trigger;
use crate::Result;

/// A trigger that runs on an HTTP request
struct HttpTrigger;

impl Trigger for HttpTrigger {
    fn listen(self: Self) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
