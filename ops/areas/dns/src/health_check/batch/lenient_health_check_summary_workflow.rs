use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves lenient health check summaries through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LenientHealthCheckSummaryCoordinator {
    pub store: Rc<dyn LenientHealthCheckSummaryStore>,
    pub validator: LenientHealthCheckSummaryValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl LenientHealthCheckSummaryCoordinator {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient health check summary {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LenientHealthCheckSummaryStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn settle(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient health check summary {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LenientHealthCheckSummaryStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.created_at > cutoff {
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

pub fn new_lenient_health_check_summary_coordinator(
    store: Rc<dyn LenientHealthCheckSummaryStore>,
    clock: Rc<dyn clock::Clock>,
) -> LenientHealthCheckSummaryCoordinator {
    LenientHealthCheckSummaryCoordinator {
        store: store.clone(),
        validator: default_lenient_health_check_summary_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
