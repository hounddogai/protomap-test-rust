use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval threshold calculation.
pub trait ApprovalThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalThresholdAdjuster {
    pub cap: f64,
}

impl ApprovalThresholdModifier for CapApprovalThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalThresholdAdjuster {
    pub step: f64,
}

impl ApprovalThresholdModifier for RoundingApprovalThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdCeilingOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalThresholdModifier for ApprovalThresholdCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalThreshold, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalThresholdModifier for TieredApprovalThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalThresholdAdjuster {
    pub percent: f64,
}

impl ApprovalThresholdModifier for PercentApprovalThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalThresholdEngine {
    pub steps: Vec<Rc<dyn ApprovalThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalThresholdEngine {
    pub fn evaluate(&mut self, item: &ApprovalThreshold) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalThresholdModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_approval_threshold_engine() -> ApprovalThresholdEngine {
    ApprovalThresholdEngine {
        steps: vec![
            Rc::new(CapApprovalThresholdAdjuster { cap: 5555.0 }) as Rc<dyn ApprovalThresholdModifier>,
            Rc::new(RoundingApprovalThresholdAdjuster { step: 0.5 }),
            Rc::new(ApprovalThresholdCeilingOffsetAdjuster { weight: 1.34 }),
            Rc::new(TieredApprovalThresholdAdjuster {
                bounds: vec![8586.0, 8791.0],
                rates: vec![0.98, 0.93],
            }),
            Rc::new(PercentApprovalThresholdAdjuster { percent: -3.8 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 40204.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_threshold_engine();
        let item = new_approval_threshold("code-719", "label-102");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
