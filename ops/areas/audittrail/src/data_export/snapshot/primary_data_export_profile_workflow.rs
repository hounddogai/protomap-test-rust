use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves primary data export profiles through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct PrimaryDataExportProfileManager {
    pub store: Rc<dyn PrimaryDataExportProfileStore>,
    pub validator: PrimaryDataExportProfileValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: PrimaryDataExportProfileEngine,
}

impl PrimaryDataExportProfileManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary data export profile {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(PrimaryDataExportProfileStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("primary data export profile {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(PrimaryDataExportProfileStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn quote(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("primary data export profile {} not found", code));
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

pub fn new_primary_data_export_profile_manager(
    store: Rc<dyn PrimaryDataExportProfileStore>,
    clock: Rc<dyn clock::Clock>,
) -> PrimaryDataExportProfileManager {
    PrimaryDataExportProfileManager {
        store: store.clone(),
        validator: default_primary_data_export_profile_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_primary_data_export_profile_engine(),
    }
}
