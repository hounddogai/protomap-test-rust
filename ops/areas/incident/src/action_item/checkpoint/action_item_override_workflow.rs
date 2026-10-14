use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves action item overrides through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ActionItemOverrideManager {
    pub store: Rc<dyn ActionItemOverrideStore>,
    pub validator: ActionItemOverrideValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl ActionItemOverrideManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("action item override {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ActionItemOverrideStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("action item override {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ActionItemOverrideStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.ends_at > cutoff {
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

pub fn new_action_item_override_manager(
    store: Rc<dyn ActionItemOverrideStore>,
    clock: Rc<dyn clock::Clock>,
) -> ActionItemOverrideManager {
    ActionItemOverrideManager {
        store: store.clone(),
        validator: default_action_item_override_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
