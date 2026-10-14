use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves overage batches through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct OverageBatchDesk {
    pub store: Rc<dyn OverageBatchStore>,
    pub validator: OverageBatchValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl OverageBatchDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("overage batch {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(OverageBatchStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("overage batch {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(OverageBatchStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_overage_batch_desk(store: Rc<dyn OverageBatchStore>, clock: Rc<dyn clock::Clock>) -> OverageBatchDesk {
    OverageBatchDesk {
        store: store.clone(),
        validator: default_overage_batch_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
