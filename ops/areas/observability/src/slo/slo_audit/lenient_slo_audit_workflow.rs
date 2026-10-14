use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves lenient slo audits through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LenientSloAuditWorkflow {
    pub store: Rc<dyn LenientSloAuditStore>,
    pub validator: LenientSloAuditValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: LenientSloAuditEngine,
}

impl LenientSloAuditWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient slo audit {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LenientSloAuditStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient slo audit {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LenientSloAuditStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("lenient slo audit {} not found", code));
        };
        let result = self.engine.price(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
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

pub fn new_lenient_slo_audit_workflow(
    store: Rc<dyn LenientSloAuditStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> LenientSloAuditWorkflow {
    LenientSloAuditWorkflow {
        store: store.clone(),
        validator: default_lenient_slo_audit_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_lenient_slo_audit_engine(flags),
    }
}
