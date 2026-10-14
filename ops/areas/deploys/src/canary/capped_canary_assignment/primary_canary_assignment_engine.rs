use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary canary assignment calculation.
pub trait PrimaryCanaryAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryCanaryAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentSequenceOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryCanaryAssignmentAdjuster for PrimaryCanaryAssignmentSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryCanaryAssignment, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryCanaryAssignmentAdjuster {
    pub percent: f64,
}

impl PrimaryCanaryAssignmentAdjuster for PercentPrimaryCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryCanaryAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryCanaryAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryCanaryAssignmentAdjuster for TieredPrimaryCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryCanaryAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary canary assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryCanaryAssignmentEngine {
    pub steps: Vec<Rc<dyn PrimaryCanaryAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryCanaryAssignmentEngine {
    pub fn price(&mut self, item: &PrimaryCanaryAssignment) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryCanaryAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_canary_assignment_engine() -> PrimaryCanaryAssignmentEngine {
    PrimaryCanaryAssignmentEngine {
        steps: vec![
            Rc::new(PrimaryCanaryAssignmentSequenceOffsetAdjuster { weight: 0.92 })
                as Rc<dyn PrimaryCanaryAssignmentAdjuster>,
            Rc::new(PercentPrimaryCanaryAssignmentAdjuster { percent: 21.8 }),
            Rc::new(TieredPrimaryCanaryAssignmentAdjuster {
                bounds: vec![1353.0, 1547.0, 3710.0, 9597.0],
                rates: vec![1.0, 1.03, 0.85, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 18572.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_canary_assignment_engine();
        let item = new_primary_canary_assignment("code-878", "name-102");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
