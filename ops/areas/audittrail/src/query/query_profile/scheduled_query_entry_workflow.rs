use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves scheduled query entries through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ScheduledQueryEntryManager {
    pub store: Rc<dyn ScheduledQueryEntryStore>,
    pub validator: ScheduledQueryEntryValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: ScheduledQueryEntryEngine,
}

impl ScheduledQueryEntryManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled query entry {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ScheduledQueryEntryStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled query entry {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ScheduledQueryEntryStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("scheduled query entry {} not found", code));
        };
        let result = self.engine.evaluate(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.updated_at > cutoff {
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

pub fn new_scheduled_query_entry_manager(
    store: Rc<dyn ScheduledQueryEntryStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> ScheduledQueryEntryManager {
    ScheduledQueryEntryManager {
        store: store.clone(),
        validator: default_scheduled_query_entry_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_scheduled_query_entry_engine(flags),
    }
}
