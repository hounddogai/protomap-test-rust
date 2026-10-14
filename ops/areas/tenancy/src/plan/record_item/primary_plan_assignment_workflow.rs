use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves primary plan assignments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PrimaryPlanAssignmentDesk {
    pub store: Rc<dyn PrimaryPlanAssignmentStore>,
    pub validator: PrimaryPlanAssignmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl PrimaryPlanAssignmentDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary plan assignment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PrimaryPlanAssignmentStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary plan assignment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PrimaryPlanAssignmentStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
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

pub fn new_primary_plan_assignment_desk(
    store: Rc<dyn PrimaryPlanAssignmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> PrimaryPlanAssignmentDesk {
    PrimaryPlanAssignmentDesk {
        store: store.clone(),
        validator: default_primary_plan_assignment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
