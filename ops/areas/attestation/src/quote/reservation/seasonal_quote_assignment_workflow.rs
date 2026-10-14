use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves seasonal quote assignments through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct SeasonalQuoteAssignmentManager {
    pub store: Rc<dyn SeasonalQuoteAssignmentStore>,
    pub validator: SeasonalQuoteAssignmentValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl SeasonalQuoteAssignmentManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("seasonal quote assignment {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(SeasonalQuoteAssignmentStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("seasonal quote assignment {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(SeasonalQuoteAssignmentStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
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

pub fn new_seasonal_quote_assignment_manager(
    store: Rc<dyn SeasonalQuoteAssignmentStore>,
    clock: Rc<dyn clock::Clock>,
) -> SeasonalQuoteAssignmentManager {
    SeasonalQuoteAssignmentManager {
        store: store.clone(),
        validator: default_seasonal_quote_assignment_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
