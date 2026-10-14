use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves adaptive federation proposals through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AdaptiveFederationProposalService {
    pub store: Rc<dyn AdaptiveFederationProposalStore>,
    pub validator: AdaptiveFederationProposalValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: AdaptiveFederationProposalEngine,
}

impl AdaptiveFederationProposalService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive federation proposal {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AdaptiveFederationProposalStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive federation proposal {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AdaptiveFederationProposalStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("adaptive federation proposal {} not found", code));
        };
        let result = self.engine.evaluate(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
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

pub fn new_adaptive_federation_proposal_service(
    store: Rc<dyn AdaptiveFederationProposalStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> AdaptiveFederationProposalService {
    AdaptiveFederationProposalService {
        store: store.clone(),
        validator: default_adaptive_federation_proposal_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_adaptive_federation_proposal_engine(flags),
    }
}
