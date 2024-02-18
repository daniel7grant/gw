use super::{Check, CheckError};
use std::{collections::HashMap, result::Result};

/// A check to watch a directory for changes.
pub struct WatchCheck;

impl Check for WatchCheck {
    fn check(&mut self, _context: &mut HashMap<String, String>) -> Result<bool, CheckError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
