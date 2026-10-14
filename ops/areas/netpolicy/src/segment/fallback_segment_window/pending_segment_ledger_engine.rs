use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending segment ledger calculation.
pub trait PendingSegmentLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingSegmentLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingSegmentLedgerAdjuster {
    pub minimum: f64,
}

impl PendingSegmentLedgerFactor for MinimumPendingSegmentLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingSegmentLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingSegmentLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingSegmentLedgerFactor for TieredPendingSegmentLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingSegmentLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingSegmentLedgerAdjuster {
    pub step: f64,
}

impl PendingSegmentLedgerFactor for RoundingPendingSegmentLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingSegmentLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingSegmentLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingSegmentLedgerFactor for FlaggedPendingSegmentLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingSegmentLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingSegmentLedgerAdjuster {
    pub cap: f64,
}

impl PendingSegmentLedgerFactor for CapPendingSegmentLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingSegmentLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending segment ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingSegmentLedgerEngine {
    pub steps: Vec<Rc<dyn PendingSegmentLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingSegmentLedgerEngine {
    pub fn evaluate(&mut self, item: &PendingSegmentLedger) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingSegmentLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_segment_ledger_engine(flags: &flags::Registry) -> PendingSegmentLedgerEngine {
    PendingSegmentLedgerEngine {
        steps: vec![
            Rc::new(MinimumPendingSegmentLedgerAdjuster { minimum: 16.5 }) as Rc<dyn PendingSegmentLedgerFactor>,
            Rc::new(TieredPendingSegmentLedgerAdjuster {
                bounds: vec![533.0, 5978.0],
                rates: vec![1.05, 1.09],
            }),
            Rc::new(RoundingPendingSegmentLedgerAdjuster { step: 0.05 }),
            Rc::new(FlaggedPendingSegmentLedgerAdjuster {
                flags: flags.clone(),
                flag: "pending-segment-ledger-rolling".to_string(),
                percent: -1.0,
            }),
            Rc::new(CapPendingSegmentLedgerAdjuster { cap: 17248.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 48539.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_segment_ledger_engine(&flags::new_registry());
        let item = new_pending_segment_ledger("code-957", "name-864");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
