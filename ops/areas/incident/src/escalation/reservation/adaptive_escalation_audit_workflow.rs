use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves adaptive escalation audits through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AdaptiveEscalationAuditManager {
    pub store: Rc<dyn AdaptiveEscalationAuditStore>,
    pub validator: AdaptiveEscalationAuditValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: AdaptiveEscalationAuditEngine,
}

impl AdaptiveEscalationAuditManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive escalation audit {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AdaptiveEscalationAuditStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive escalation audit {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AdaptiveEscalationAuditStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("adaptive escalation audit {} not found", code));
        };
        let result = self.engine.compute(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
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

pub fn new_adaptive_escalation_audit_manager(
    store: Rc<dyn AdaptiveEscalationAuditStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> AdaptiveEscalationAuditManager {
    AdaptiveEscalationAuditManager {
        store: store.clone(),
        validator: default_adaptive_escalation_audit_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_adaptive_escalation_audit_engine(flags),
    }
}
