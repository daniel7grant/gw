use super::Check;
use crate::Result;

/// A check to watch a directory for changes
pub struct WatchCheck;

impl Check for WatchCheck {
    fn check(&mut self) -> Result<bool> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
