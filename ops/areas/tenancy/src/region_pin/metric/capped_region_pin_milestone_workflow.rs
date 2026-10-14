use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves capped region pin milestones through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct CappedRegionPinMilestoneService {
    pub store: Rc<dyn CappedRegionPinMilestoneStore>,
    pub validator: CappedRegionPinMilestoneValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: CappedRegionPinMilestoneEngine,
}

impl CappedRegionPinMilestoneService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("capped region pin milestone {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(CappedRegionPinMilestoneStatus::Warming)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is warming", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("capped region pin milestone {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(CappedRegionPinMilestoneStatus::Stopped)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is stopped: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("capped region pin milestone {} not found", code));
        };
        let result = self.engine.price(&item);
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

pub fn new_capped_region_pin_milestone_service(
    store: Rc<dyn CappedRegionPinMilestoneStore>,
    clock: Rc<dyn clock::Clock>,
) -> CappedRegionPinMilestoneService {
    CappedRegionPinMilestoneService {
        store: store.clone(),
        validator: default_capped_region_pin_milestone_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_capped_region_pin_milestone_engine(),
    }
}
