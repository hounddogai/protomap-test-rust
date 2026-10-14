use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves manual flow log assignments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ManualFlowLogAssignmentWorkflow {
    pub store: Rc<dyn ManualFlowLogAssignmentStore>,
    pub validator: ManualFlowLogAssignmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: ManualFlowLogAssignmentEngine,
}

impl ManualFlowLogAssignmentWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("manual flow log assignment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ManualFlowLogAssignmentStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("manual flow log assignment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ManualFlowLogAssignmentStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("manual flow log assignment {} not found", code));
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

pub fn new_manual_flow_log_assignment_workflow(
    store: Rc<dyn ManualFlowLogAssignmentStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> ManualFlowLogAssignmentWorkflow {
    ManualFlowLogAssignmentWorkflow {
        store: store.clone(),
        validator: default_manual_flow_log_assignment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_manual_flow_log_assignment_engine(flags),
    }
}
