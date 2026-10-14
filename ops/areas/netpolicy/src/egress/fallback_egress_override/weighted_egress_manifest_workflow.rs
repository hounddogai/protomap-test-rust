use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves weighted egress manifests through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct WeightedEgressManifestService {
    pub store: Rc<dyn WeightedEgressManifestStore>,
    pub validator: WeightedEgressManifestValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: WeightedEgressManifestEngine,
}

impl WeightedEgressManifestService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("weighted egress manifest {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(WeightedEgressManifestStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("weighted egress manifest {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(WeightedEgressManifestStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("weighted egress manifest {} not found", code));
        };
        let result = self.engine.price(&item);
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

pub fn new_weighted_egress_manifest_service(
    store: Rc<dyn WeightedEgressManifestStore>,
    clock: Rc<dyn clock::Clock>,
) -> WeightedEgressManifestService {
    WeightedEgressManifestService {
        store: store.clone(),
        validator: default_weighted_egress_manifest_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_weighted_egress_manifest_engine(),
    }
}
