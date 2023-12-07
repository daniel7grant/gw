use std::collections::HashMap;

use super::Repository;

pub struct TestRepository {
    pub directory: String,
    pub updates: bool,
}

impl Repository for TestRepository {
    fn open(directory: &str) -> Result<Self, String> {
        Ok(TestRepository {
            directory: String::from(directory),
            updates: false,
        })
    }

    fn get_directory(self: &Self) -> String {
        self.directory.clone()
    }

    fn get_envs(self: &Self) -> HashMap<String, String> {
        HashMap::default()
    }

    fn check_for_updates(self: &Self) -> Result<bool, String> {
        Ok(self.updates)
    }

    fn pull_updates(self: &mut Self) -> Result<bool, String> {
        let did_update = self.updates;
        self.updates = false;
        Ok(did_update)
    }
}
