use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves release profiles through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ReleaseProfileWorkflow {
    pub store: Rc<dyn ReleaseProfileStore>,
    pub validator: ReleaseProfileValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: ReleaseProfileEngine,
}

impl ReleaseProfileWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("release profile {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ReleaseProfileStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("release profile {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ReleaseProfileStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("release profile {} not found", code));
        };
        let result = self.engine.run(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            let problems = self.validator.validate(&item);
            if (problems.len() as i64) == 0 {
                continue;
            }
            self.note(&format!("{} needs attention: {}", item.code, problems.join(", ")));
            changed += 1;
        }
        changed
    }
}

pub fn new_release_profile_workflow(
    store: Rc<dyn ReleaseProfileStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> ReleaseProfileWorkflow {
    ReleaseProfileWorkflow {
        store: store.clone(),
        validator: default_release_profile_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_release_profile_engine(flags),
    }
}
