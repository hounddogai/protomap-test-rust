use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves evidence rollups through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct EvidenceRollupManager {
    pub store: Rc<dyn EvidenceRollupStore>,
    pub validator: EvidenceRollupValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: EvidenceRollupEngine,
}

impl EvidenceRollupManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("evidence rollup {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(EvidenceRollupStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("evidence rollup {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(EvidenceRollupStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("evidence rollup {} not found", code));
        };
        let result = self.engine.evaluate(&item);
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

pub fn new_evidence_rollup_manager(
    store: Rc<dyn EvidenceRollupStore>,
    clock: Rc<dyn clock::Clock>,
) -> EvidenceRollupManager {
    EvidenceRollupManager {
        store: store.clone(),
        validator: default_evidence_rollup_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_evidence_rollup_engine(),
    }
}
