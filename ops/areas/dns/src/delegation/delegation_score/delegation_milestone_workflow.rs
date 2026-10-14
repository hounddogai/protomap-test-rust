use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves delegation milestones through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct DelegationMilestoneDesk {
    pub store: Rc<dyn DelegationMilestoneStore>,
    pub validator: DelegationMilestoneValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl DelegationMilestoneDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("delegation milestone {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(DelegationMilestoneStatus::Warming)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is warming", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("delegation milestone {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(DelegationMilestoneStatus::Stopped)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is stopped: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
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

pub fn new_delegation_milestone_desk(
    store: Rc<dyn DelegationMilestoneStore>,
    clock: Rc<dyn clock::Clock>,
) -> DelegationMilestoneDesk {
    DelegationMilestoneDesk {
        store: store.clone(),
        validator: default_delegation_milestone_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
