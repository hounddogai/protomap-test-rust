use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves capped peering records through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct CappedPeeringRecordWorkflow {
    pub store: Rc<dyn CappedPeeringRecordStore>,
    pub validator: CappedPeeringRecordValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: CappedPeeringRecordEngine,
}

impl CappedPeeringRecordWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("capped peering record {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(CappedPeeringRecordStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("capped peering record {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(CappedPeeringRecordStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("capped peering record {} not found", code));
        };
        let result = self.engine.run(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_capped_peering_record_workflow(
    store: Rc<dyn CappedPeeringRecordStore>,
    clock: Rc<dyn clock::Clock>,
) -> CappedPeeringRecordWorkflow {
    CappedPeeringRecordWorkflow {
        store: store.clone(),
        validator: default_capped_peering_record_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_capped_peering_record_engine(),
    }
}
