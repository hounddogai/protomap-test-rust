use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves allowlist segments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AllowlistSegmentDesk {
    pub store: Rc<dyn AllowlistSegmentStore>,
    pub validator: AllowlistSegmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl AllowlistSegmentDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("allowlist segment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AllowlistSegmentStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("allowlist segment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AllowlistSegmentStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_allowlist_segment_desk(
    store: Rc<dyn AllowlistSegmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> AllowlistSegmentDesk {
    AllowlistSegmentDesk {
        store: store.clone(),
        validator: default_allowlist_segment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
