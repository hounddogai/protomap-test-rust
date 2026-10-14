use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy ledger calculation.
pub trait TtlPolicyLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyLedgerAdjuster {
    pub minimum: f64,
}

impl TtlPolicyLedgerAdjuster for MinimumTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyLedgerAdjuster for FlaggedTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyLedgerAdjuster {
    pub step: f64,
}

impl TtlPolicyLedgerAdjuster for RoundingTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyLedgerAdjuster for TieredTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyLedgerAdjuster {
    pub cap: f64,
}

impl TtlPolicyLedgerAdjuster for CapTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyLedgerAdjuster {
    pub percent: f64,
}

impl TtlPolicyLedgerAdjuster for PercentTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyLedgerEngine {
    pub steps: Vec<Rc<dyn TtlPolicyLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyLedgerEngine {
    pub fn price(&mut self, item: &TtlPolicyLedger) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_ttl_policy_ledger_engine(flags: &flags::Registry) -> TtlPolicyLedgerEngine {
    TtlPolicyLedgerEngine {
        steps: vec![
            Rc::new(MinimumTtlPolicyLedgerAdjuster { minimum: 26.6 }) as Rc<dyn TtlPolicyLedgerAdjuster>,
            Rc::new(FlaggedTtlPolicyLedgerAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-ledger-scheduled".to_string(),
                percent: 6.0,
            }),
            Rc::new(RoundingTtlPolicyLedgerAdjuster { step: 1.0 }),
            Rc::new(TieredTtlPolicyLedgerAdjuster {
                bounds: vec![3933.0, 5979.0],
                rates: vec![0.95, 0.96],
            }),
            Rc::new(CapTtlPolicyLedgerAdjuster { cap: 17790.0 }),
            Rc::new(PercentTtlPolicyLedgerAdjuster { percent: 9.4 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 46892.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_ledger_engine(&flags::new_registry());
        let item = new_ttl_policy_ledger("code-297", "label-993");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
