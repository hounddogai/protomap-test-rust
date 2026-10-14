use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves overage baselines through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct OverageBaselineWorkflow {
    pub store: Rc<dyn OverageBaselineStore>,
    pub validator: OverageBaselineValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl OverageBaselineWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("overage baseline {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(OverageBaselineStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("overage baseline {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(OverageBaselineStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
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

pub fn new_overage_baseline_workflow(
    store: Rc<dyn OverageBaselineStore>,
    clock: Rc<dyn clock::Clock>,
) -> OverageBaselineWorkflow {
    OverageBaselineWorkflow {
        store: store.clone(),
        validator: default_overage_baseline_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
