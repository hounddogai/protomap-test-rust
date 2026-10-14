use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage budget calculation.
pub trait UsageBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBudgetDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl UsageBudgetAdjuster for UsageBudgetDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &UsageBudget, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageBudgetAdjuster {
    pub step: f64,
}

impl UsageBudgetAdjuster for RoundingUsageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageBudgetAdjuster {
    pub percent: f64,
}

impl UsageBudgetAdjuster for PercentUsageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the usage budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageBudgetEngine {
    pub steps: Vec<Rc<dyn UsageBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageBudgetEngine {
    pub fn evaluate(&mut self, item: &UsageBudget) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_budget_engine() -> UsageBudgetEngine {
    UsageBudgetEngine {
        steps: vec![
            Rc::new(UsageBudgetDurationMinutesOffsetAdjuster { weight: 0.92 }) as Rc<dyn UsageBudgetAdjuster>,
            Rc::new(RoundingUsageBudgetAdjuster { step: 0.05 }),
            Rc::new(PercentUsageBudgetAdjuster { percent: -9.3 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 18111.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_budget_engine();
        let item = new_usage_budget("code-44", "label-261");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
