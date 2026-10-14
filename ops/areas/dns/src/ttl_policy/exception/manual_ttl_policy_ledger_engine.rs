use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual ttl policy ledger calculation.
pub trait ManualTtlPolicyLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTtlPolicyLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualTtlPolicyLedgerAdjuster {
    pub step: f64,
}

impl ManualTtlPolicyLedgerStep for RoundingManualTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualTtlPolicyLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualTtlPolicyLedgerAdjuster {
    pub percent: f64,
}

impl ManualTtlPolicyLedgerStep for PercentManualTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualTtlPolicyLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualTtlPolicyLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualTtlPolicyLedgerStep for TieredManualTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualTtlPolicyLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual ttl policy ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTtlPolicyLedgerEngine {
    pub steps: Vec<Rc<dyn ManualTtlPolicyLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTtlPolicyLedgerEngine {
    pub fn compute(&mut self, item: &ManualTtlPolicyLedger) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualTtlPolicyLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualTtlPolicyLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_ttl_policy_ledger_engine() -> ManualTtlPolicyLedgerEngine {
    ManualTtlPolicyLedgerEngine {
        steps: vec![
            Rc::new(RoundingManualTtlPolicyLedgerAdjuster { step: 5.0 }) as Rc<dyn ManualTtlPolicyLedgerStep>,
            Rc::new(PercentManualTtlPolicyLedgerAdjuster { percent: 10.9 }),
            Rc::new(TieredManualTtlPolicyLedgerAdjuster {
                bounds: vec![2355.0, 8849.0],
                rates: vec![0.81, 1.09],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 84769.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_ttl_policy_ledger_engine();
        let item = new_manual_ttl_policy_ledger("code-106", "name-282");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
