use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual release ledger calculation.
pub trait ManualReleaseLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualReleaseLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryManualReleaseLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualReleaseLedgerAdjuster for CategoryManualReleaseLedgerAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ManualReleaseLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualReleaseLedgerAdjuster {
    pub percent: f64,
}

impl ManualReleaseLedgerAdjuster for PercentManualReleaseLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualReleaseLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualReleaseLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualReleaseLedgerAdjuster for FlaggedManualReleaseLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualReleaseLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseLedgerStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl ManualReleaseLedgerAdjuster for ManualReleaseLedgerStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &ManualReleaseLedger, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the manual release ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualReleaseLedgerEngine {
    pub steps: Vec<Rc<dyn ManualReleaseLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualReleaseLedgerEngine {
    pub fn price(&mut self, item: &ManualReleaseLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualReleaseLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_release_ledger_engine(flags: &flags::Registry) -> ManualReleaseLedgerEngine {
    ManualReleaseLedgerEngine {
        steps: vec![
            Rc::new(CategoryManualReleaseLedgerAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.17), ("retail".to_string(), 0.73)]),
            }) as Rc<dyn ManualReleaseLedgerAdjuster>,
            Rc::new(PercentManualReleaseLedgerAdjuster { percent: 23.5 }),
            Rc::new(FlaggedManualReleaseLedgerAdjuster {
                flags: flags.clone(),
                flag: "manual-release-ledger-default".to_string(),
                percent: 9.8,
            }),
            Rc::new(ManualReleaseLedgerStartsAtOffsetAdjuster { weight: 1.9 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 19681.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_release_ledger_engine(&flags::new_registry());
        let item = new_manual_release_ledger("code-57", "title-627");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
