use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive canary assignment calculation.
pub trait AdaptiveCanaryAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveCanaryAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryAssignmentScoreOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveCanaryAssignmentStep for AdaptiveCanaryAssignmentScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveCanaryAssignment, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveCanaryAssignmentAdjuster {
    pub percent: f64,
}

impl AdaptiveCanaryAssignmentStep for PercentAdaptiveCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveCanaryAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveCanaryAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveCanaryAssignmentStep for FlaggedAdaptiveCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveCanaryAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveCanaryAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveCanaryAssignmentStep for TieredAdaptiveCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveCanaryAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive canary assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveCanaryAssignmentEngine {
    pub steps: Vec<Rc<dyn AdaptiveCanaryAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveCanaryAssignmentEngine {
    pub fn price(&mut self, item: &AdaptiveCanaryAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveCanaryAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveCanaryAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_canary_assignment_engine(flags: &flags::Registry) -> AdaptiveCanaryAssignmentEngine {
    AdaptiveCanaryAssignmentEngine {
        steps: vec![
            Rc::new(AdaptiveCanaryAssignmentScoreOffsetAdjuster { weight: 1.83 })
                as Rc<dyn AdaptiveCanaryAssignmentStep>,
            Rc::new(PercentAdaptiveCanaryAssignmentAdjuster { percent: -18.1 }),
            Rc::new(FlaggedAdaptiveCanaryAssignmentAdjuster {
                flags: flags.clone(),
                flag: "adaptive-canary-assignment-pending".to_string(),
                percent: 3.9,
            }),
            Rc::new(TieredAdaptiveCanaryAssignmentAdjuster {
                bounds: vec![4839.0, 8292.0],
                rates: vec![1.01, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 76906.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_canary_assignment_engine(&flags::new_registry());
        let item = new_adaptive_canary_assignment("code-574", "label-170");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
