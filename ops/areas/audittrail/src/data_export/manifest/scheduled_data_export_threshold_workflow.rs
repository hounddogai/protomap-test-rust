use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves scheduled data export thresholds through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportThresholdService {
    pub store: Rc<dyn ScheduledDataExportThresholdStore>,
    pub validator: ScheduledDataExportThresholdValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: ScheduledDataExportThresholdEngine,
}

impl ScheduledDataExportThresholdService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled data export threshold {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ScheduledDataExportThresholdStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("scheduled data export threshold {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ScheduledDataExportThresholdStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn score(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("scheduled data export threshold {} not found", code));
        };
        let result = self.engine.price(&item);
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

pub fn new_scheduled_data_export_threshold_service(
    store: Rc<dyn ScheduledDataExportThresholdStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> ScheduledDataExportThresholdService {
    ScheduledDataExportThresholdService {
        store: store.clone(),
        validator: default_scheduled_data_export_threshold_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_scheduled_data_export_threshold_engine(flags),
    }
}
