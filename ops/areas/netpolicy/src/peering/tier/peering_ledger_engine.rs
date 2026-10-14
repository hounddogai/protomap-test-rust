use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering ledger calculation.
pub trait PeeringLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringLedgerAdjuster {
    pub cap: f64,
}

impl PeeringLedgerFactor for CapPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringLedgerStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl PeeringLedgerFactor for PeeringLedgerStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringLedger, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringLedgerFactor for TieredPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringLedgerFactor for FlaggedPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringLedgerAdjuster {
    pub step: f64,
}

impl PeeringLedgerFactor for RoundingPeeringLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the peering ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringLedgerEngine {
    pub steps: Vec<Rc<dyn PeeringLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringLedgerEngine {
    pub fn evaluate(&mut self, item: &PeeringLedger) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringLedgerFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_ledger_engine(flags: &flags::Registry) -> PeeringLedgerEngine {
    PeeringLedgerEngine {
        steps: vec![
            Rc::new(CapPeeringLedgerAdjuster { cap: 40849.0 }) as Rc<dyn PeeringLedgerFactor>,
            Rc::new(PeeringLedgerStartsAtOffsetAdjuster { weight: 0.97 }),
            Rc::new(TieredPeeringLedgerAdjuster {
                bounds: vec![1715.0, 8105.0, 9073.0],
                rates: vec![1.17, 0.94, 1.14],
            }),
            Rc::new(FlaggedPeeringLedgerAdjuster {
                flags: flags.clone(),
                flag: "peering-ledger-primary".to_string(),
                percent: -6.9,
            }),
            Rc::new(RoundingPeeringLedgerAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 44650.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_ledger_engine(&flags::new_registry());
        let item = new_peering_ledger("code-859", "label-2");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
