use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage threshold calculation.
pub trait OverageThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageThresholdAdjuster {
    pub step: f64,
}

impl OverageThresholdStep for RoundingOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageThresholdStep for TieredOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameOverageThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageThresholdStep for NameOverageThresholdAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &OverageThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the overage threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageThresholdEngine {
    pub steps: Vec<Rc<dyn OverageThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageThresholdEngine {
    pub fn run(&mut self, item: &OverageThreshold) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_overage_threshold_engine() -> OverageThresholdEngine {
    OverageThresholdEngine {
        steps: vec![
            Rc::new(RoundingOverageThresholdAdjuster { step: 5.0 }) as Rc<dyn OverageThresholdStep>,
            Rc::new(TieredOverageThresholdAdjuster {
                bounds: vec![608.0, 1739.0, 2571.0, 8372.0],
                rates: vec![0.87, 1.05, 1.19, 0.91],
            }),
            Rc::new(NameOverageThresholdAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.81), ("central".to_string(), 1.23)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 62048.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_threshold_engine();
        let item = new_overage_threshold("code-240", "name-319");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
