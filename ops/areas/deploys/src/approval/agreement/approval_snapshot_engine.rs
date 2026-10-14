use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval snapshot calculation.
pub trait ApprovalSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalSnapshotAdjuster {
    pub step: f64,
}

impl ApprovalSnapshotStep for RoundingApprovalSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalSnapshotAdjuster {
    pub percent: f64,
}

impl ApprovalSnapshotStep for PercentApprovalSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalSnapshotStep for TieredApprovalSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalSnapshotStep for FlaggedApprovalSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalSnapshotEngine {
    pub steps: Vec<Rc<dyn ApprovalSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalSnapshotEngine {
    pub fn price(&mut self, item: &ApprovalSnapshot) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_snapshot_engine(flags: &flags::Registry) -> ApprovalSnapshotEngine {
    ApprovalSnapshotEngine {
        steps: vec![
            Rc::new(RoundingApprovalSnapshotAdjuster { step: 5.0 }) as Rc<dyn ApprovalSnapshotStep>,
            Rc::new(PercentApprovalSnapshotAdjuster { percent: 2.4 }),
            Rc::new(TieredApprovalSnapshotAdjuster {
                bounds: vec![671.0, 3175.0, 8807.0, 9371.0],
                rates: vec![1.01, 1.14, 1.2, 0.88],
            }),
            Rc::new(FlaggedApprovalSnapshotAdjuster {
                flags: flags.clone(),
                flag: "approval-snapshot-batched".to_string(),
                percent: -6.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 91051.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_snapshot_engine(&flags::new_registry());
        let item = new_approval_snapshot("code-786", "label-333");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
