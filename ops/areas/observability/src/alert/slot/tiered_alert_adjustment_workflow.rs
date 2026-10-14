use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves tiered alert adjustments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct TieredAlertAdjustmentService {
    pub store: Rc<dyn TieredAlertAdjustmentStore>,
    pub validator: TieredAlertAdjustmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl TieredAlertAdjustmentService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn open(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered alert adjustment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(TieredAlertAdjustmentStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered alert adjustment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(TieredAlertAdjustmentStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
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

pub fn new_tiered_alert_adjustment_service(
    store: Rc<dyn TieredAlertAdjustmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> TieredAlertAdjustmentService {
    TieredAlertAdjustmentService {
        store: store.clone(),
        validator: default_tiered_alert_adjustment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
