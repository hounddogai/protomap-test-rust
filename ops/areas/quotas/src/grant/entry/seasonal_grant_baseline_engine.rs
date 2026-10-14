use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal grant baseline calculation.
pub trait SeasonalGrantBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalGrantBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalGrantBaselineAdjuster {
    pub step: f64,
}

impl SeasonalGrantBaselineStep for RoundingSeasonalGrantBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalGrantBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalGrantBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalGrantBaselineStep for TieredSeasonalGrantBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalGrantBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantBaselineWeightOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalGrantBaselineStep for SeasonalGrantBaselineWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalGrantBaseline, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the seasonal grant baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalGrantBaselineEngine {
    pub steps: Vec<Rc<dyn SeasonalGrantBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalGrantBaselineEngine {
    pub fn evaluate(&mut self, item: &SeasonalGrantBaseline) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalGrantBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalGrantBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_grant_baseline_engine() -> SeasonalGrantBaselineEngine {
    SeasonalGrantBaselineEngine {
        steps: vec![
            Rc::new(RoundingSeasonalGrantBaselineAdjuster { step: 0.25 }) as Rc<dyn SeasonalGrantBaselineStep>,
            Rc::new(TieredSeasonalGrantBaselineAdjuster {
                bounds: vec![4136.0, 5516.0],
                rates: vec![1.17, 0.89],
            }),
            Rc::new(SeasonalGrantBaselineWeightOffsetAdjuster { weight: 1.59 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 57360.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_grant_baseline_engine();
        let item = new_seasonal_grant_baseline("code-508", "title-494");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
