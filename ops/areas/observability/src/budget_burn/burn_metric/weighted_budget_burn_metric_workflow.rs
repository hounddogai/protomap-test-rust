use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves weighted budget burn metrics through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnMetricWorkflow {
    pub store: Rc<dyn WeightedBudgetBurnMetricStore>,
    pub validator: WeightedBudgetBurnMetricValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: WeightedBudgetBurnMetricEngine,
}

impl WeightedBudgetBurnMetricWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn submit(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("weighted budget burn metric {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(WeightedBudgetBurnMetricStatus::Warming)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is warming", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("weighted budget burn metric {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(WeightedBudgetBurnMetricStatus::Stopped)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is stopped: {}", code, reason));
        Ok(())
    }

    pub fn estimate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("weighted budget burn metric {} not found", code));
        };
        let result = self.engine.price(&item);
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

pub fn new_weighted_budget_burn_metric_workflow(
    store: Rc<dyn WeightedBudgetBurnMetricStore>,
    clock: Rc<dyn clock::Clock>,
) -> WeightedBudgetBurnMetricWorkflow {
    WeightedBudgetBurnMetricWorkflow {
        store: store.clone(),
        validator: default_weighted_budget_burn_metric_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_weighted_budget_burn_metric_engine(),
    }
}
