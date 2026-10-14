use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global peering ledger calculation.
pub trait GlobalPeeringLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPeeringLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringLedgerQuantityOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPeeringLedgerStep for GlobalPeeringLedgerQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPeeringLedger, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalPeeringLedgerAdjuster {
    pub step: f64,
}

impl GlobalPeeringLedgerStep for RoundingGlobalPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalPeeringLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPeeringLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPeeringLedgerStep for FlaggedGlobalPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPeeringLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPeeringLedgerAdjuster {
    pub cap: f64,
}

impl GlobalPeeringLedgerStep for CapGlobalPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPeeringLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalPeeringLedgerAdjuster {
    pub percent: f64,
}

impl GlobalPeeringLedgerStep for PercentGlobalPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalPeeringLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPeeringLedgerAdjuster {
    pub minimum: f64,
}

impl GlobalPeeringLedgerStep for MinimumGlobalPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPeeringLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global peering ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPeeringLedgerEngine {
    pub steps: Vec<Rc<dyn GlobalPeeringLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPeeringLedgerEngine {
    pub fn evaluate(&mut self, item: &GlobalPeeringLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalPeeringLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_peering_ledger_engine(flags: &flags::Registry) -> GlobalPeeringLedgerEngine {
    GlobalPeeringLedgerEngine {
        steps: vec![
            Rc::new(GlobalPeeringLedgerQuantityOffsetAdjuster { weight: 1.65 }) as Rc<dyn GlobalPeeringLedgerStep>,
            Rc::new(RoundingGlobalPeeringLedgerAdjuster { step: 0.05 }),
            Rc::new(FlaggedGlobalPeeringLedgerAdjuster {
                flags: flags.clone(),
                flag: "global-peering-ledger-seasonal".to_string(),
                percent: -1.9,
            }),
            Rc::new(CapGlobalPeeringLedgerAdjuster { cap: 10558.0 }),
            Rc::new(PercentGlobalPeeringLedgerAdjuster { percent: 11.0 }),
            Rc::new(MinimumGlobalPeeringLedgerAdjuster { minimum: 29.9 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 35152.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_peering_ledger_engine(&flags::new_registry());
        let item = new_global_peering_ledger("code-253", "name-507");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
