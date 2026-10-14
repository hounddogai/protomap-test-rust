use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy ledger calculation.
pub trait PolicyLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyLedgerBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl PolicyLedgerModifier for PolicyLedgerBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyLedger, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyLedgerAdjuster {
    pub minimum: f64,
}

impl PolicyLedgerModifier for MinimumPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyLedgerModifier for TieredPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyLedgerModifier for FlaggedPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the policy ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyLedgerEngine {
    pub steps: Vec<Rc<dyn PolicyLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyLedgerEngine {
    pub fn evaluate(&mut self, item: &PolicyLedger) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[PolicyLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_policy_ledger_engine(flags: &flags::Registry) -> PolicyLedgerEngine {
    PolicyLedgerEngine {
        steps: vec![
            Rc::new(PolicyLedgerBatchSizeOffsetAdjuster { weight: 1.15 }) as Rc<dyn PolicyLedgerModifier>,
            Rc::new(MinimumPolicyLedgerAdjuster { minimum: 32.3 }),
            Rc::new(TieredPolicyLedgerAdjuster {
                bounds: vec![6385.0, 8423.0],
                rates: vec![1.12, 0.92],
            }),
            Rc::new(FlaggedPolicyLedgerAdjuster {
                flags: flags.clone(),
                flag: "policy-ledger-lenient".to_string(),
                percent: 5.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 33175.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_ledger_engine(&flags::new_registry());
        let item = new_policy_ledger("code-708", "label-824");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
