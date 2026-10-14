use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval policy calculation.
pub trait ApprovalPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPolicySequenceOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalPolicyAdjuster for ApprovalPolicySequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalPolicy, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalPolicyAdjuster for TieredApprovalPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalPolicyAdjuster {
    pub step: f64,
}

impl ApprovalPolicyAdjuster for RoundingApprovalPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalPolicyAdjuster {
    pub cap: f64,
}

impl ApprovalPolicyAdjuster for CapApprovalPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalPolicyAdjuster {
    pub percent: f64,
}

impl ApprovalPolicyAdjuster for PercentApprovalPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalPolicyAdjuster {
    pub minimum: f64,
}

impl ApprovalPolicyAdjuster for MinimumApprovalPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the approval policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalPolicyEngine {
    pub steps: Vec<Rc<dyn ApprovalPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalPolicyEngine {
    pub fn compute(&mut self, item: &ApprovalPolicy) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_approval_policy_engine() -> ApprovalPolicyEngine {
    ApprovalPolicyEngine {
        steps: vec![
            Rc::new(ApprovalPolicySequenceOffsetAdjuster { weight: 0.2 }) as Rc<dyn ApprovalPolicyAdjuster>,
            Rc::new(TieredApprovalPolicyAdjuster {
                bounds: vec![1380.0, 2212.0, 4384.0],
                rates: vec![1.19, 1.13, 1.15],
            }),
            Rc::new(RoundingApprovalPolicyAdjuster { step: 5.0 }),
            Rc::new(CapApprovalPolicyAdjuster { cap: 45768.0 }),
            Rc::new(PercentApprovalPolicyAdjuster { percent: -21.1 }),
            Rc::new(MinimumApprovalPolicyAdjuster { minimum: 20.1 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 19147.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_policy_engine();
        let item = new_approval_policy("code-114", "label-763");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
