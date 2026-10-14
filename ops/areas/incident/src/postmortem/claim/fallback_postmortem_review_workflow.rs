use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback postmortem reviews through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemReviewCoordinator {
    pub store: Rc<dyn FallbackPostmortemReviewStore>,
    pub validator: FallbackPostmortemReviewValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: FallbackPostmortemReviewEngine,
}

impl FallbackPostmortemReviewCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback postmortem review {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackPostmortemReviewStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback postmortem review {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackPostmortemReviewStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("fallback postmortem review {} not found", code));
        };
        let result = self.engine.evaluate(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.starts_at > cutoff {
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

pub fn new_fallback_postmortem_review_coordinator(
    store: Rc<dyn FallbackPostmortemReviewStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackPostmortemReviewCoordinator {
    FallbackPostmortemReviewCoordinator {
        store: store.clone(),
        validator: default_fallback_postmortem_review_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_fallback_postmortem_review_engine(),
    }
}
