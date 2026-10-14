use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves regional quote metrics through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct RegionalQuoteMetricWorkflow {
    pub store: Rc<dyn RegionalQuoteMetricStore>,
    pub validator: RegionalQuoteMetricValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl RegionalQuoteMetricWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("regional quote metric {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(RegionalQuoteMetricStatus::Pending)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is pending", code));
        Ok(())
    }

    pub fn complete(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("regional quote metric {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(RegionalQuoteMetricStatus::Closed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is closed: {}", code, reason));
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

pub fn new_regional_quote_metric_workflow(
    store: Rc<dyn RegionalQuoteMetricStore>,
    clock: Rc<dyn clock::Clock>,
) -> RegionalQuoteMetricWorkflow {
    RegionalQuoteMetricWorkflow {
        store: store.clone(),
        validator: default_regional_quote_metric_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
