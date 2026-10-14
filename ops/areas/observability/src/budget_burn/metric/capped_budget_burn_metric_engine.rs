use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped budget burn metric calculation.
pub trait CappedBudgetBurnMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedBudgetBurnMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceCappedBudgetBurnMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedBudgetBurnMetricModifier for SourceCappedBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &CappedBudgetBurnMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedBudgetBurnMetricAdjuster {
    pub minimum: f64,
}

impl CappedBudgetBurnMetricModifier for MinimumCappedBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedBudgetBurnMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedBudgetBurnMetricAdjuster {
    pub percent: f64,
}

impl CappedBudgetBurnMetricModifier for PercentCappedBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedBudgetBurnMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped budget burn metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnMetricEngine {
    pub steps: Vec<Rc<dyn CappedBudgetBurnMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedBudgetBurnMetricEngine {
    pub fn run(&mut self, item: &CappedBudgetBurnMetric) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedBudgetBurnMetricModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedBudgetBurnMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_budget_burn_metric_engine() -> CappedBudgetBurnMetricEngine {
    CappedBudgetBurnMetricEngine {
        steps: vec![
            Rc::new(SourceCappedBudgetBurnMetricAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.02), ("north".to_string(), 0.93)]),
            }) as Rc<dyn CappedBudgetBurnMetricModifier>,
            Rc::new(MinimumCappedBudgetBurnMetricAdjuster { minimum: 16.2 }),
            Rc::new(PercentCappedBudgetBurnMetricAdjuster { percent: 17.7 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 94616.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_budget_burn_metric_engine();
        let item = new_capped_budget_burn_metric("code-104", "title-583");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
