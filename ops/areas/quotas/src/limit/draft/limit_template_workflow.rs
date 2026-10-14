use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves limit templates through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LimitTemplateWorkflow {
    pub store: Rc<dyn LimitTemplateStore>,
    pub validator: LimitTemplateValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl LimitTemplateWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("limit template {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LimitTemplateStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("limit template {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LimitTemplateStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
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

pub fn new_limit_template_workflow(
    store: Rc<dyn LimitTemplateStore>,
    clock: Rc<dyn clock::Clock>,
) -> LimitTemplateWorkflow {
    LimitTemplateWorkflow {
        store: store.clone(),
        validator: default_limit_template_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
