use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending region pin ledger calculation.
pub trait PendingRegionPinLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRegionPinLedger, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRegionPinLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRegionPinLedgerStep for FlaggedPendingRegionPinLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRegionPinLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingRegionPinLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRegionPinLedgerStep for NamePendingRegionPinLedgerAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingRegionPinLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinLedgerMinimumOffsetAdjuster {
    pub weight: f64,
}

impl PendingRegionPinLedgerStep for PendingRegionPinLedgerMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &PendingRegionPinLedger, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingRegionPinLedgerAdjuster {
    pub cap: f64,
}

impl PendingRegionPinLedgerStep for CapPendingRegionPinLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingRegionPinLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending region pin ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRegionPinLedgerEngine {
    pub steps: Vec<Rc<dyn PendingRegionPinLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRegionPinLedgerEngine {
    pub fn evaluate(&mut self, item: &PendingRegionPinLedger) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingRegionPinLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingRegionPinLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_region_pin_ledger_engine(flags: &flags::Registry) -> PendingRegionPinLedgerEngine {
    PendingRegionPinLedgerEngine {
        steps: vec![
            Rc::new(FlaggedPendingRegionPinLedgerAdjuster {
                flags: flags.clone(),
                flag: "pending-region-pin-ledger-weighted".to_string(),
                percent: -7.9,
            }) as Rc<dyn PendingRegionPinLedgerStep>,
            Rc::new(NamePendingRegionPinLedgerAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.92)]),
            }),
            Rc::new(PendingRegionPinLedgerMinimumOffsetAdjuster { weight: 0.12 }),
            Rc::new(CapPendingRegionPinLedgerAdjuster { cap: 5739.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 63567.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_region_pin_ledger_engine(&flags::new_registry());
        let item = new_pending_region_pin_ledger("code-905", "name-83");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
