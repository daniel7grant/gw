use super::Action;
use crate::Result;

/// An action to run a custom shell script.
pub struct ScriptAction;

impl Action for ScriptAction {
    fn run(&self) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
