use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn snapshot calculation.
pub trait BudgetBurnSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnSnapshotAdjuster for FlaggedBudgetBurnSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnSnapshotAdjuster {
    pub minimum: f64,
}

impl BudgetBurnSnapshotAdjuster for MinimumBudgetBurnSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBudgetBurnSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnSnapshotAdjuster for LabelBudgetBurnSnapshotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BudgetBurnSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnSnapshotAdjuster {
    pub step: f64,
}

impl BudgetBurnSnapshotAdjuster for RoundingBudgetBurnSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the budget burn snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnSnapshotEngine {
    pub steps: Vec<Rc<dyn BudgetBurnSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnSnapshotEngine {
    pub fn evaluate(&mut self, item: &BudgetBurnSnapshot) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_budget_burn_snapshot_engine(flags: &flags::Registry) -> BudgetBurnSnapshotEngine {
    BudgetBurnSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedBudgetBurnSnapshotAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-snapshot-batched".to_string(),
                percent: -1.5,
            }) as Rc<dyn BudgetBurnSnapshotAdjuster>,
            Rc::new(MinimumBudgetBurnSnapshotAdjuster { minimum: 8.1 }),
            Rc::new(LabelBudgetBurnSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.03),
                    ("west".to_string(), 0.8),
                    ("east".to_string(), 0.73),
                ]),
            }),
            Rc::new(RoundingBudgetBurnSnapshotAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 99960.2,
    }
}
