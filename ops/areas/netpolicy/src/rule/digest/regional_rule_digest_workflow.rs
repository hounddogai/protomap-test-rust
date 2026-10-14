use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves regional rule digests through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RegionalRuleDigestManager {
    pub store: Rc<dyn RegionalRuleDigestStore>,
    pub validator: RegionalRuleDigestValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: RegionalRuleDigestEngine,
}

impl RegionalRuleDigestManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("regional rule digest {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RegionalRuleDigestStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("regional rule digest {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RegionalRuleDigestStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("regional rule digest {} not found", code));
        };
        let result = self.engine.run(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
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

pub fn new_regional_rule_digest_manager(
    store: Rc<dyn RegionalRuleDigestStore>,
    clock: Rc<dyn clock::Clock>,
) -> RegionalRuleDigestManager {
    RegionalRuleDigestManager {
        store: store.clone(),
        validator: default_regional_rule_digest_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_regional_rule_digest_engine(),
    }
}
