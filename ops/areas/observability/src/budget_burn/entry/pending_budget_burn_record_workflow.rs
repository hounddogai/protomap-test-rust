use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves pending budget burn records through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PendingBudgetBurnRecordWorkflow {
    pub store: Rc<dyn PendingBudgetBurnRecordStore>,
    pub validator: PendingBudgetBurnRecordValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: PendingBudgetBurnRecordEngine,
}

impl PendingBudgetBurnRecordWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("pending budget burn record {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PendingBudgetBurnRecordStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("pending budget burn record {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PendingBudgetBurnRecordStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("pending budget burn record {} not found", code));
        };
        let result = self.engine.run(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
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

pub fn new_pending_budget_burn_record_workflow(
    store: Rc<dyn PendingBudgetBurnRecordStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> PendingBudgetBurnRecordWorkflow {
    PendingBudgetBurnRecordWorkflow {
        store: store.clone(),
        validator: default_pending_budget_burn_record_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_pending_budget_burn_record_engine(flags),
    }
}
