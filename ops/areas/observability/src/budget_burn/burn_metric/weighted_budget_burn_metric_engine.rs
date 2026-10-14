use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted budget burn metric calculation.
pub trait WeightedBudgetBurnMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedBudgetBurnMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedBudgetBurnMetricAdjuster {
    pub cap: f64,
}

impl WeightedBudgetBurnMetricStep for CapWeightedBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedBudgetBurnMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedBudgetBurnMetricAdjuster {
    pub step: f64,
}

impl WeightedBudgetBurnMetricStep for RoundingWeightedBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedBudgetBurnMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedBudgetBurnMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedBudgetBurnMetricStep for TieredWeightedBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedBudgetBurnMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedBudgetBurnMetricAdjuster {
    pub minimum: f64,
}

impl WeightedBudgetBurnMetricStep for MinimumWeightedBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedBudgetBurnMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetricRevisionOffsetAdjuster {
    pub weight: f64,
}

impl WeightedBudgetBurnMetricStep for WeightedBudgetBurnMetricRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedBudgetBurnMetric, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedBudgetBurnMetricAdjuster {
    pub percent: f64,
}

impl WeightedBudgetBurnMetricStep for PercentWeightedBudgetBurnMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedBudgetBurnMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted budget burn metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnMetricEngine {
    pub steps: Vec<Rc<dyn WeightedBudgetBurnMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedBudgetBurnMetricEngine {
    pub fn price(&mut self, item: &WeightedBudgetBurnMetric) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedBudgetBurnMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedBudgetBurnMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_budget_burn_metric_engine() -> WeightedBudgetBurnMetricEngine {
    WeightedBudgetBurnMetricEngine {
        steps: vec![
            Rc::new(CapWeightedBudgetBurnMetricAdjuster { cap: 12786.0 }) as Rc<dyn WeightedBudgetBurnMetricStep>,
            Rc::new(RoundingWeightedBudgetBurnMetricAdjuster { step: 1.0 }),
            Rc::new(TieredWeightedBudgetBurnMetricAdjuster {
                bounds: vec![6423.0, 9574.0],
                rates: vec![1.18, 1.1],
            }),
            Rc::new(MinimumWeightedBudgetBurnMetricAdjuster { minimum: 22.3 }),
            Rc::new(WeightedBudgetBurnMetricRevisionOffsetAdjuster { weight: 0.32 }),
            Rc::new(PercentWeightedBudgetBurnMetricAdjuster { percent: -23.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 40357.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_budget_burn_metric_engine();
        let item = new_weighted_budget_burn_metric("code-628", "title-794");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
