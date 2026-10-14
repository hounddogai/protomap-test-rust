use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves nonce checkpoints through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct NonceCheckpointWorkflow {
    pub store: Rc<dyn NonceCheckpointStore>,
    pub validator: NonceCheckpointValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl NonceCheckpointWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("nonce checkpoint {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(NonceCheckpointStatus::Warming)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is warming", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("nonce checkpoint {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(NonceCheckpointStatus::Stopped)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is stopped: {}", code, reason));
        Ok(())
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

pub fn new_nonce_checkpoint_workflow(
    store: Rc<dyn NonceCheckpointStore>,
    clock: Rc<dyn clock::Clock>,
) -> NonceCheckpointWorkflow {
    NonceCheckpointWorkflow {
        store: store.clone(),
        validator: default_nonce_checkpoint_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
