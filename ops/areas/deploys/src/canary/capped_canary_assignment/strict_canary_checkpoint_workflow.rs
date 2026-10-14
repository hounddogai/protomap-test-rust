use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves strict canary checkpoints through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct StrictCanaryCheckpointManager {
    pub store: Rc<dyn StrictCanaryCheckpointStore>,
    pub validator: StrictCanaryCheckpointValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: StrictCanaryCheckpointEngine,
}

impl StrictCanaryCheckpointManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict canary checkpoint {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(StrictCanaryCheckpointStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict canary checkpoint {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(StrictCanaryCheckpointStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("strict canary checkpoint {} not found", code));
        };
        let result = self.engine.price(&item);
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

pub fn new_strict_canary_checkpoint_manager(
    store: Rc<dyn StrictCanaryCheckpointStore>,
    clock: Rc<dyn clock::Clock>,
) -> StrictCanaryCheckpointManager {
    StrictCanaryCheckpointManager {
        store: store.clone(),
        validator: default_strict_canary_checkpoint_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_strict_canary_checkpoint_engine(),
    }
}
