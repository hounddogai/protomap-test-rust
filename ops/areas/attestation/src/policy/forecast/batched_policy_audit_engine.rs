use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched policy audit calculation.
pub trait BatchedPolicyAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPolicyAudit, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedPolicyAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedPolicyAuditAdjuster for FlaggedBatchedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedPolicyAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedPolicyAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedPolicyAuditAdjuster for TieredBatchedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedPolicyAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedPolicyAuditAdjuster {
    pub step: f64,
}

impl BatchedPolicyAuditAdjuster for RoundingBatchedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedPolicyAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedPolicyAuditAdjuster {
    pub percent: f64,
}

impl BatchedPolicyAuditAdjuster for PercentBatchedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedPolicyAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedPolicyAuditAdjuster {
    pub cap: f64,
}

impl BatchedPolicyAuditAdjuster for CapBatchedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedPolicyAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedPolicyAuditAdjuster {
    pub minimum: f64,
}

impl BatchedPolicyAuditAdjuster for MinimumBatchedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedPolicyAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched policy audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPolicyAuditEngine {
    pub steps: Vec<Rc<dyn BatchedPolicyAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPolicyAuditEngine {
    pub fn evaluate(&mut self, item: &BatchedPolicyAudit) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedPolicyAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_policy_audit_engine(flags: &flags::Registry) -> BatchedPolicyAuditEngine {
    BatchedPolicyAuditEngine {
        steps: vec![
            Rc::new(FlaggedBatchedPolicyAuditAdjuster {
                flags: flags.clone(),
                flag: "batched-policy-audit-pending".to_string(),
                percent: -3.1,
            }) as Rc<dyn BatchedPolicyAuditAdjuster>,
            Rc::new(TieredBatchedPolicyAuditAdjuster {
                bounds: vec![1854.0, 2305.0, 6090.0, 7218.0],
                rates: vec![1.19, 1.04, 0.84, 1.01],
            }),
            Rc::new(RoundingBatchedPolicyAuditAdjuster { step: 0.05 }),
            Rc::new(PercentBatchedPolicyAuditAdjuster { percent: -21.8 }),
            Rc::new(CapBatchedPolicyAuditAdjuster { cap: 16986.0 }),
            Rc::new(MinimumBatchedPolicyAuditAdjuster { minimum: 42.6 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 93987.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_policy_audit_engine(&flags::new_registry());
        let item = new_batched_policy_audit("code-313", "label-634");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
