use super::Check;
use crate::Result;

/// A check to fetch and pull a local git repository.
pub struct GitCheck;

impl Check for GitCheck {
    fn check(&mut self) -> Result<bool> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
