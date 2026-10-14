use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves regional dns record snapshots through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RegionalDnsRecordSnapshotCoordinator {
    pub store: Rc<dyn RegionalDnsRecordSnapshotStore>,
    pub validator: RegionalDnsRecordSnapshotValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: RegionalDnsRecordSnapshotEngine,
}

impl RegionalDnsRecordSnapshotCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("regional dns record snapshot {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RegionalDnsRecordSnapshotStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("regional dns record snapshot {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RegionalDnsRecordSnapshotStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("regional dns record snapshot {} not found", code));
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

pub fn new_regional_dns_record_snapshot_coordinator(
    store: Rc<dyn RegionalDnsRecordSnapshotStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> RegionalDnsRecordSnapshotCoordinator {
    RegionalDnsRecordSnapshotCoordinator {
        store: store.clone(),
        validator: default_regional_dns_record_snapshot_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_regional_dns_record_snapshot_engine(flags),
    }
}
