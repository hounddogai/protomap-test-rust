use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves primary binding segments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PrimaryBindingSegmentManager {
    pub store: Rc<dyn PrimaryBindingSegmentStore>,
    pub validator: PrimaryBindingSegmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl PrimaryBindingSegmentManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary binding segment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PrimaryBindingSegmentStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary binding segment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PrimaryBindingSegmentStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
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

pub fn new_primary_binding_segment_manager(
    store: Rc<dyn PrimaryBindingSegmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> PrimaryBindingSegmentManager {
    PrimaryBindingSegmentManager {
        store: store.clone(),
        validator: default_primary_binding_segment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
