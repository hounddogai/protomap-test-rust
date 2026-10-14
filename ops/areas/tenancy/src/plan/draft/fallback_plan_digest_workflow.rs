use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback plan digests through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackPlanDigestCoordinator {
    pub store: Rc<dyn FallbackPlanDigestStore>,
    pub validator: FallbackPlanDigestValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FallbackPlanDigestCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback plan digest {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackPlanDigestStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback plan digest {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackPlanDigestStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
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

pub fn new_fallback_plan_digest_coordinator(
    store: Rc<dyn FallbackPlanDigestStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackPlanDigestCoordinator {
    FallbackPlanDigestCoordinator {
        store: store.clone(),
        validator: default_fallback_plan_digest_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
