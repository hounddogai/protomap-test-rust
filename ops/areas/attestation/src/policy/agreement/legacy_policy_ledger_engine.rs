use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy policy ledger calculation.
pub trait LegacyPolicyLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyPolicyLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyPolicyLedgerAdjuster {
    pub step: f64,
}

impl LegacyPolicyLedgerStep for RoundingLegacyPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyPolicyLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyPolicyLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyPolicyLedgerStep for TieredLegacyPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyPolicyLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyPolicyLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyPolicyLedgerStep for FlaggedLegacyPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyPolicyLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy policy ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyPolicyLedgerEngine {
    pub steps: Vec<Rc<dyn LegacyPolicyLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyPolicyLedgerEngine {
    pub fn run(&mut self, item: &LegacyPolicyLedger) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyPolicyLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyPolicyLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_policy_ledger_engine(flags: &flags::Registry) -> LegacyPolicyLedgerEngine {
    LegacyPolicyLedgerEngine {
        steps: vec![
            Rc::new(RoundingLegacyPolicyLedgerAdjuster { step: 0.05 }) as Rc<dyn LegacyPolicyLedgerStep>,
            Rc::new(TieredLegacyPolicyLedgerAdjuster {
                bounds: vec![6044.0, 7842.0, 9043.0],
                rates: vec![1.01, 0.91, 1.06],
            }),
            Rc::new(FlaggedLegacyPolicyLedgerAdjuster {
                flags: flags.clone(),
                flag: "legacy-policy-ledger-manual".to_string(),
                percent: 8.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 84006.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_policy_ledger_engine(&flags::new_registry());
        let item = new_legacy_policy_ledger("code-10", "label-872");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
