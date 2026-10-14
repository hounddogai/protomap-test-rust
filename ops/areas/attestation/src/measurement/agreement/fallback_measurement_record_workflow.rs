use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback measurement records through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackMeasurementRecordService {
    pub store: Rc<dyn FallbackMeasurementRecordStore>,
    pub validator: FallbackMeasurementRecordValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FallbackMeasurementRecordService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback measurement record {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackMeasurementRecordStatus::InReview)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is in review", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback measurement record {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackMeasurementRecordStatus::Archived)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is archived: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.starts_at > cutoff {
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

pub fn new_fallback_measurement_record_service(
    store: Rc<dyn FallbackMeasurementRecordStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackMeasurementRecordService {
    FallbackMeasurementRecordService {
        store: store.clone(),
        validator: default_fallback_measurement_record_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
