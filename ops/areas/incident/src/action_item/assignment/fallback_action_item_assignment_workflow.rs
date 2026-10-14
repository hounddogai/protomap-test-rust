use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback action item assignments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackActionItemAssignmentService {
    pub store: Rc<dyn FallbackActionItemAssignmentStore>,
    pub validator: FallbackActionItemAssignmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FallbackActionItemAssignmentService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback action item assignment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackActionItemAssignmentStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback action item assignment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackActionItemAssignmentStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
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

pub fn new_fallback_action_item_assignment_service(
    store: Rc<dyn FallbackActionItemAssignmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackActionItemAssignmentService {
    FallbackActionItemAssignmentService {
        store: store.clone(),
        validator: default_fallback_action_item_assignment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
