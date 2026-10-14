use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves seasonal timeline digests through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct SeasonalTimelineDigestManager {
    pub store: Rc<dyn SeasonalTimelineDigestStore>,
    pub validator: SeasonalTimelineDigestValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: SeasonalTimelineDigestEngine,
}

impl SeasonalTimelineDigestManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("seasonal timeline digest {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(SeasonalTimelineDigestStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("seasonal timeline digest {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(SeasonalTimelineDigestStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("seasonal timeline digest {} not found", code));
        };
        let result = self.engine.compute(&item);
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

pub fn new_seasonal_timeline_digest_manager(
    store: Rc<dyn SeasonalTimelineDigestStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> SeasonalTimelineDigestManager {
    SeasonalTimelineDigestManager {
        store: store.clone(),
        validator: default_seasonal_timeline_digest_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_seasonal_timeline_digest_engine(flags),
    }
}
