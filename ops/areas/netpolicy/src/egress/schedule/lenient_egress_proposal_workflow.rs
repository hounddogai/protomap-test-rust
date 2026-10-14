use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves lenient egress proposals through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LenientEgressProposalDesk {
    pub store: Rc<dyn LenientEgressProposalStore>,
    pub validator: LenientEgressProposalValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl LenientEgressProposalDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient egress proposal {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LenientEgressProposalStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient egress proposal {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LenientEgressProposalStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
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

pub fn new_lenient_egress_proposal_desk(
    store: Rc<dyn LenientEgressProposalStore>,
    clock: Rc<dyn clock::Clock>,
) -> LenientEgressProposalDesk {
    LenientEgressProposalDesk {
        store: store.clone(),
        validator: default_lenient_egress_proposal_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
