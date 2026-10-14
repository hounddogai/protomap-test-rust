use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves global timeline proposals through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct GlobalTimelineProposalService {
    pub store: Rc<dyn GlobalTimelineProposalStore>,
    pub validator: GlobalTimelineProposalValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: GlobalTimelineProposalEngine,
}

impl GlobalTimelineProposalService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global timeline proposal {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(GlobalTimelineProposalStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("global timeline proposal {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(GlobalTimelineProposalStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("global timeline proposal {} not found", code));
        };
        let result = self.engine.run(&item);
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

pub fn new_global_timeline_proposal_service(
    store: Rc<dyn GlobalTimelineProposalStore>,
    clock: Rc<dyn clock::Clock>,
) -> GlobalTimelineProposalService {
    GlobalTimelineProposalService {
        store: store.clone(),
        validator: default_global_timeline_proposal_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_global_timeline_proposal_engine(),
    }
}
