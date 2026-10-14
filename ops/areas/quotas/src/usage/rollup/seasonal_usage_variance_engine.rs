use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal usage variance calculation.
pub trait SeasonalUsageVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalUsageVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalUsageVarianceAdjuster {
    pub percent: f64,
}

impl SeasonalUsageVarianceModifier for PercentSeasonalUsageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalUsageVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageVarianceAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalUsageVarianceModifier for SeasonalUsageVarianceAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalUsageVariance, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalUsageVarianceAdjuster {
    pub step: f64,
}

impl SeasonalUsageVarianceModifier for RoundingSeasonalUsageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalUsageVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal usage variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalUsageVarianceEngine {
    pub steps: Vec<Rc<dyn SeasonalUsageVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalUsageVarianceEngine {
    pub fn price(&mut self, item: &SeasonalUsageVariance) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalUsageVarianceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalUsageVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_usage_variance_engine() -> SeasonalUsageVarianceEngine {
    SeasonalUsageVarianceEngine {
        steps: vec![
            Rc::new(PercentSeasonalUsageVarianceAdjuster { percent: -8.5 }) as Rc<dyn SeasonalUsageVarianceModifier>,
            Rc::new(SeasonalUsageVarianceAttemptsOffsetAdjuster { weight: 0.42 }),
            Rc::new(RoundingSeasonalUsageVarianceAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 93902.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_usage_variance_engine();
        let item = new_seasonal_usage_variance("code-705", "name-141");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
