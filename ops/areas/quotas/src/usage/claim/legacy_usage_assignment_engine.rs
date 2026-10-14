use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy usage assignment calculation.
pub trait LegacyUsageAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyUsageAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyUsageAssignmentAdjuster {
    pub percent: f64,
}

impl LegacyUsageAssignmentAdjuster for PercentLegacyUsageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyUsageAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAssignmentVersionOffsetAdjuster {
    pub weight: f64,
}

impl LegacyUsageAssignmentAdjuster for LegacyUsageAssignmentVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyUsageAssignment, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyUsageAssignmentAdjuster {
    pub step: f64,
}

impl LegacyUsageAssignmentAdjuster for RoundingLegacyUsageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyUsageAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyUsageAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyUsageAssignmentAdjuster for TieredLegacyUsageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyUsageAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyUsageAssignmentAdjuster {
    pub minimum: f64,
}

impl LegacyUsageAssignmentAdjuster for MinimumLegacyUsageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyUsageAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy usage assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyUsageAssignmentEngine {
    pub steps: Vec<Rc<dyn LegacyUsageAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyUsageAssignmentEngine {
    pub fn run(&mut self, item: &LegacyUsageAssignment) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[LegacyUsageAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_usage_assignment_engine() -> LegacyUsageAssignmentEngine {
    LegacyUsageAssignmentEngine {
        steps: vec![
            Rc::new(PercentLegacyUsageAssignmentAdjuster { percent: 5.5 }) as Rc<dyn LegacyUsageAssignmentAdjuster>,
            Rc::new(LegacyUsageAssignmentVersionOffsetAdjuster { weight: 1.01 }),
            Rc::new(RoundingLegacyUsageAssignmentAdjuster { step: 0.5 }),
            Rc::new(TieredLegacyUsageAssignmentAdjuster {
                bounds: vec![5550.0, 7201.0, 8015.0, 8360.0],
                rates: vec![1.12, 1.2, 0.88, 1.08],
            }),
            Rc::new(MinimumLegacyUsageAssignmentAdjuster { minimum: 13.1 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 45321.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_usage_assignment_engine();
        let item = new_legacy_usage_assignment("code-72", "name-663");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
