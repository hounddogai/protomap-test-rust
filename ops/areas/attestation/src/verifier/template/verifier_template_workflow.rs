use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves verifier templates through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct VerifierTemplateWorkflow {
    pub store: Rc<dyn VerifierTemplateStore>,
    pub validator: VerifierTemplateValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl VerifierTemplateWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("verifier template {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(VerifierTemplateStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("verifier template {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(VerifierTemplateStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
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

pub fn new_verifier_template_workflow(
    store: Rc<dyn VerifierTemplateStore>,
    clock: Rc<dyn clock::Clock>,
) -> VerifierTemplateWorkflow {
    VerifierTemplateWorkflow {
        store: store.clone(),
        validator: default_verifier_template_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
