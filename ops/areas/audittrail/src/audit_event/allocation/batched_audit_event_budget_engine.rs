use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched audit event budget calculation.
pub trait BatchedAuditEventBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAuditEventBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventBudgetCostOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAuditEventBudgetAdjuster for BatchedAuditEventBudgetCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAuditEventBudget, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedAuditEventBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedAuditEventBudgetAdjuster for FlaggedBatchedAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedAuditEventBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterBatchedAuditEventBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedAuditEventBudgetAdjuster for ClusterBatchedAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &BatchedAuditEventBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedAuditEventBudgetAdjuster {
    pub step: f64,
}

impl BatchedAuditEventBudgetAdjuster for RoundingBatchedAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedAuditEventBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedAuditEventBudgetAdjuster {
    pub minimum: f64,
}

impl BatchedAuditEventBudgetAdjuster for MinimumBatchedAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedAuditEventBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched audit event budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAuditEventBudgetEngine {
    pub steps: Vec<Rc<dyn BatchedAuditEventBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAuditEventBudgetEngine {
    pub fn compute(&mut self, item: &BatchedAuditEventBudget) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedAuditEventBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_audit_event_budget_engine(flags: &flags::Registry) -> BatchedAuditEventBudgetEngine {
    BatchedAuditEventBudgetEngine {
        steps: vec![
            Rc::new(BatchedAuditEventBudgetCostOffsetAdjuster { weight: 1.03 })
                as Rc<dyn BatchedAuditEventBudgetAdjuster>,
            Rc::new(FlaggedBatchedAuditEventBudgetAdjuster {
                flags: flags.clone(),
                flag: "batched-audit-event-budget-capped".to_string(),
                percent: 4.9,
            }),
            Rc::new(ClusterBatchedAuditEventBudgetAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.26), ("central".to_string(), 1.39)]),
            }),
            Rc::new(RoundingBatchedAuditEventBudgetAdjuster { step: 0.1 }),
            Rc::new(MinimumBatchedAuditEventBudgetAdjuster { minimum: 30.3 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 31383.8,
    }
}
