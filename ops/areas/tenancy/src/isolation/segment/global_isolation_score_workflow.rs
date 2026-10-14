use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves global isolation scores through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct GlobalIsolationScoreService {
    pub store: Rc<dyn GlobalIsolationScoreStore>,
    pub validator: GlobalIsolationScoreValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl GlobalIsolationScoreService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global isolation score {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(GlobalIsolationScoreStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global isolation score {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(GlobalIsolationScoreStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
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

pub fn new_global_isolation_score_service(
    store: Rc<dyn GlobalIsolationScoreStore>,
    clock: Rc<dyn clock::Clock>,
) -> GlobalIsolationScoreService {
    GlobalIsolationScoreService {
        store: store.clone(),
        validator: default_global_isolation_score_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
