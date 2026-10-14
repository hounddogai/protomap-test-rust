use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves adaptive usage segments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct AdaptiveUsageSegmentManager {
    pub store: Rc<dyn AdaptiveUsageSegmentStore>,
    pub validator: AdaptiveUsageSegmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl AdaptiveUsageSegmentManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive usage segment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(AdaptiveUsageSegmentStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("adaptive usage segment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(AdaptiveUsageSegmentStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn audit(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.ends_at > cutoff {
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

pub fn new_adaptive_usage_segment_manager(
    store: Rc<dyn AdaptiveUsageSegmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> AdaptiveUsageSegmentManager {
    AdaptiveUsageSegmentManager {
        store: store.clone(),
        validator: default_adaptive_usage_segment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
