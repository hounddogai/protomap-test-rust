use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves rolling zone snapshots through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RollingZoneSnapshotManager {
    pub store: Rc<dyn RollingZoneSnapshotStore>,
    pub validator: RollingZoneSnapshotValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl RollingZoneSnapshotManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("rolling zone snapshot {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RollingZoneSnapshotStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("rolling zone snapshot {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RollingZoneSnapshotStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
        Ok(())
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

pub fn new_rolling_zone_snapshot_manager(
    store: Rc<dyn RollingZoneSnapshotStore>,
    clock: Rc<dyn clock::Clock>,
) -> RollingZoneSnapshotManager {
    RollingZoneSnapshotManager {
        store: store.clone(),
        validator: default_rolling_zone_snapshot_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
