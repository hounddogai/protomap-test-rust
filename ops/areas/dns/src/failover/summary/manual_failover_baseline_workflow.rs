use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves manual failover baselines through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ManualFailoverBaselineWorkflow {
    pub store: Rc<dyn ManualFailoverBaselineStore>,
    pub validator: ManualFailoverBaselineValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl ManualFailoverBaselineWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("manual failover baseline {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ManualFailoverBaselineStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("manual failover baseline {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ManualFailoverBaselineStatus::Closed)
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
            if item.created_at > cutoff {
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

pub fn new_manual_failover_baseline_workflow(
    store: Rc<dyn ManualFailoverBaselineStore>,
    clock: Rc<dyn clock::Clock>,
) -> ManualFailoverBaselineWorkflow {
    ManualFailoverBaselineWorkflow {
        store: store.clone(),
        validator: default_manual_failover_baseline_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
