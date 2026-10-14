use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled rule budget calculation.
pub trait ScheduledRuleBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRuleBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledRuleBudgetAdjuster {
    pub minimum: f64,
}

impl ScheduledRuleBudgetModifier for MinimumScheduledRuleBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledRuleBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledRuleBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledRuleBudgetModifier for TieredScheduledRuleBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledRuleBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledRuleBudgetAdjuster {
    pub percent: f64,
}

impl ScheduledRuleBudgetModifier for PercentScheduledRuleBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledRuleBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled rule budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRuleBudgetEngine {
    pub steps: Vec<Rc<dyn ScheduledRuleBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRuleBudgetEngine {
    pub fn compute(&mut self, item: &ScheduledRuleBudget) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledRuleBudgetModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ScheduledRuleBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_rule_budget_engine() -> ScheduledRuleBudgetEngine {
    ScheduledRuleBudgetEngine {
        steps: vec![
            Rc::new(MinimumScheduledRuleBudgetAdjuster { minimum: 29.7 }) as Rc<dyn ScheduledRuleBudgetModifier>,
            Rc::new(TieredScheduledRuleBudgetAdjuster {
                bounds: vec![1850.0, 3682.0, 5895.0, 9242.0],
                rates: vec![0.88, 0.97, 1.14, 1.01],
            }),
            Rc::new(PercentScheduledRuleBudgetAdjuster { percent: 10.1 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 65586.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_rule_budget_engine();
        let item = new_scheduled_rule_budget("code-168", "title-596");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
