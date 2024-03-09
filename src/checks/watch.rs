use super::{Check, CheckError};
use crate::context::Context;
use std::result::Result;

/// A check to watch a directory for changes.
pub struct WatchCheck;

impl Check for WatchCheck {
    fn check(&mut self, _context: &mut Context) -> Result<bool, CheckError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
