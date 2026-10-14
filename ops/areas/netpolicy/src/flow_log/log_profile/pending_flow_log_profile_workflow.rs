use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves pending flow log profiles through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PendingFlowLogProfileService {
    pub store: Rc<dyn PendingFlowLogProfileStore>,
    pub validator: PendingFlowLogProfileValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: PendingFlowLogProfileEngine,
}

impl PendingFlowLogProfileService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("pending flow log profile {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PendingFlowLogProfileStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("pending flow log profile {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PendingFlowLogProfileStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("pending flow log profile {} not found", code));
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

pub fn new_pending_flow_log_profile_service(
    store: Rc<dyn PendingFlowLogProfileStore>,
    clock: Rc<dyn clock::Clock>,
) -> PendingFlowLogProfileService {
    PendingFlowLogProfileService {
        store: store.clone(),
        validator: default_pending_flow_log_profile_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_pending_flow_log_profile_engine(),
    }
}
