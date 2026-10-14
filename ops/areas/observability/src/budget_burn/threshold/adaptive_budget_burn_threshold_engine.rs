use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive budget burn threshold calculation.
pub trait AdaptiveBudgetBurnThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveBudgetBurnThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveBudgetBurnThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveBudgetBurnThresholdModifier for TieredAdaptiveBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveBudgetBurnThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryAdaptiveBudgetBurnThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveBudgetBurnThresholdModifier for CountryAdaptiveBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &AdaptiveBudgetBurnThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBudgetBurnThresholdScoreOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveBudgetBurnThresholdModifier for AdaptiveBudgetBurnThresholdScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveBudgetBurnThreshold, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the adaptive budget burn threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveBudgetBurnThresholdEngine {
    pub steps: Vec<Rc<dyn AdaptiveBudgetBurnThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveBudgetBurnThresholdEngine {
    pub fn run(&mut self, item: &AdaptiveBudgetBurnThreshold) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveBudgetBurnThresholdModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveBudgetBurnThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_budget_burn_threshold_engine() -> AdaptiveBudgetBurnThresholdEngine {
    AdaptiveBudgetBurnThresholdEngine {
        steps: vec![
            Rc::new(TieredAdaptiveBudgetBurnThresholdAdjuster {
                bounds: vec![2023.0, 3833.0, 7168.0],
                rates: vec![0.96, 1.15, 1.15],
            }) as Rc<dyn AdaptiveBudgetBurnThresholdModifier>,
            Rc::new(CountryAdaptiveBudgetBurnThresholdAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.08)]),
            }),
            Rc::new(AdaptiveBudgetBurnThresholdScoreOffsetAdjuster { weight: 0.75 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 57884.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_budget_burn_threshold_engine();
        let item = new_adaptive_budget_burn_threshold("code-273", "label-157");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
