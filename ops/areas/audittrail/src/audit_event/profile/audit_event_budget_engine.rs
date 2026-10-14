use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event budget calculation.
pub trait AuditEventBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAuditEventBudgetAdjuster {
    pub percent: f64,
}

impl AuditEventBudgetAdjuster for PercentAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AuditEventBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAuditEventBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventBudgetAdjuster for LabelAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AuditEventBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventBudgetAdjuster for FlaggedAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventBudgetAdjuster {
    pub minimum: f64,
}

impl AuditEventBudgetAdjuster for MinimumAuditEventBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBudgetQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventBudgetAdjuster for AuditEventBudgetQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventBudget, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the audit event budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventBudgetEngine {
    pub steps: Vec<Rc<dyn AuditEventBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventBudgetEngine {
    pub fn evaluate(&mut self, item: &AuditEventBudget) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_audit_event_budget_engine(flags: &flags::Registry) -> AuditEventBudgetEngine {
    AuditEventBudgetEngine {
        steps: vec![
            Rc::new(PercentAuditEventBudgetAdjuster { percent: 5.5 }) as Rc<dyn AuditEventBudgetAdjuster>,
            Rc::new(LabelAuditEventBudgetAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.02)]),
            }),
            Rc::new(FlaggedAuditEventBudgetAdjuster {
                flags: flags.clone(),
                flag: "audit-event-budget-tiered".to_string(),
                percent: -0.5,
            }),
            Rc::new(MinimumAuditEventBudgetAdjuster { minimum: 9.0 }),
            Rc::new(AuditEventBudgetQuotaUnitsOffsetAdjuster { weight: 1.65 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 71238.5,
    }
}
