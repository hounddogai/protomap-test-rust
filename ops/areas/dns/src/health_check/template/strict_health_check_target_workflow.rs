use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves strict health check targets through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct StrictHealthCheckTargetService {
    pub store: Rc<dyn StrictHealthCheckTargetStore>,
    pub validator: StrictHealthCheckTargetValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl StrictHealthCheckTargetService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict health check target {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(StrictHealthCheckTargetStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict health check target {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(StrictHealthCheckTargetStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.expires_at > cutoff {
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

pub fn new_strict_health_check_target_service(
    store: Rc<dyn StrictHealthCheckTargetStore>,
    clock: Rc<dyn clock::Clock>,
) -> StrictHealthCheckTargetService {
    StrictHealthCheckTargetService {
        store: store.clone(),
        validator: default_strict_health_check_target_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
