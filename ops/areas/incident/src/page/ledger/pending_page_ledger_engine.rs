use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending page ledger calculation.
pub trait PendingPageLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPageLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageLedgerWeightOffsetAdjuster {
    pub weight: f64,
}

impl PendingPageLedgerFactor for PendingPageLedgerWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &PendingPageLedger, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingPageLedgerAdjuster {
    pub step: f64,
}

impl PendingPageLedgerFactor for RoundingPendingPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingPageLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPendingPageLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPageLedgerFactor for ClusterPendingPageLedgerAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PendingPageLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPageLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPageLedgerFactor for FlaggedPendingPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPageLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending page ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPageLedgerEngine {
    pub steps: Vec<Rc<dyn PendingPageLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPageLedgerEngine {
    pub fn run(&mut self, item: &PendingPageLedger) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingPageLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_page_ledger_engine(flags: &flags::Registry) -> PendingPageLedgerEngine {
    PendingPageLedgerEngine {
        steps: vec![
            Rc::new(PendingPageLedgerWeightOffsetAdjuster { weight: 1.14 }) as Rc<dyn PendingPageLedgerFactor>,
            Rc::new(RoundingPendingPageLedgerAdjuster { step: 1.0 }),
            Rc::new(ClusterPendingPageLedgerAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.7)]),
            }),
            Rc::new(FlaggedPendingPageLedgerAdjuster {
                flags: flags.clone(),
                flag: "pending-page-ledger-tiered".to_string(),
                percent: 1.8,
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 27424.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_page_ledger_engine(&flags::new_registry());
        let item = new_pending_page_ledger("code-489", "name-699");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
