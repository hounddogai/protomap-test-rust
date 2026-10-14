use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default budget burn metric calculation.
pub trait DefaultBudgetBurnMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultBudgetBurnMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnMetricFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl DefaultBudgetBurnMetricFactor for DefaultBudgetBurnMetricFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultBudgetBurnMetric, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultBudgetBurnMetricAdjuster {
    pub step: f64,
}

impl DefaultBudgetBurnMetricFactor for RoundingDefaultBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultBudgetBurnMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultBudgetBurnMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultBudgetBurnMetricFactor for LabelDefaultBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultBudgetBurnMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default budget burn metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultBudgetBurnMetricEngine {
    pub steps: Vec<Rc<dyn DefaultBudgetBurnMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultBudgetBurnMetricEngine {
    pub fn evaluate(&mut self, item: &DefaultBudgetBurnMetric) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultBudgetBurnMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultBudgetBurnMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_budget_burn_metric_engine() -> DefaultBudgetBurnMetricEngine {
    DefaultBudgetBurnMetricEngine {
        steps: vec![
            Rc::new(DefaultBudgetBurnMetricFloorValueOffsetAdjuster { weight: 1.81 })
                as Rc<dyn DefaultBudgetBurnMetricFactor>,
            Rc::new(RoundingDefaultBudgetBurnMetricAdjuster { step: 5.0 }),
            Rc::new(LabelDefaultBudgetBurnMetricAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.3), ("central".to_string(), 1.21)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 19583.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_budget_burn_metric_engine();
        let item = new_default_budget_burn_metric("code-117", "label-907");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
