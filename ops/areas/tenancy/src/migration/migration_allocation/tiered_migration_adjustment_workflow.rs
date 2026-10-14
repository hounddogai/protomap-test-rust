use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves tiered migration adjustments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct TieredMigrationAdjustmentCoordinator {
    pub store: Rc<dyn TieredMigrationAdjustmentStore>,
    pub validator: TieredMigrationAdjustmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: TieredMigrationAdjustmentEngine,
}

impl TieredMigrationAdjustmentCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered migration adjustment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(TieredMigrationAdjustmentStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("tiered migration adjustment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(TieredMigrationAdjustmentStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("tiered migration adjustment {} not found", code));
        };
        let result = self.engine.evaluate(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
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

pub fn new_tiered_migration_adjustment_coordinator(
    store: Rc<dyn TieredMigrationAdjustmentStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> TieredMigrationAdjustmentCoordinator {
    TieredMigrationAdjustmentCoordinator {
        store: store.clone(),
        validator: default_tiered_migration_adjustment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_tiered_migration_adjustment_engine(flags),
    }
}
