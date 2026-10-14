use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual rollout audit calculation.
pub trait ManualRolloutAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRolloutAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualRolloutAuditAdjuster {
    pub percent: f64,
}

impl ManualRolloutAuditStep for PercentManualRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualRolloutAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelManualRolloutAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualRolloutAuditStep for LabelManualRolloutAuditAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ManualRolloutAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRolloutAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRolloutAuditStep for FlaggedManualRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRolloutAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualRolloutAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualRolloutAuditStep for TieredManualRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualRolloutAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualRolloutAuditAdjuster {
    pub minimum: f64,
}

impl ManualRolloutAuditStep for MinimumManualRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualRolloutAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRolloutAuditQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ManualRolloutAuditStep for ManualRolloutAuditQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ManualRolloutAudit, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the manual rollout audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRolloutAuditEngine {
    pub steps: Vec<Rc<dyn ManualRolloutAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRolloutAuditEngine {
    pub fn evaluate(&mut self, item: &ManualRolloutAudit) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualRolloutAuditStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualRolloutAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_rollout_audit_engine(flags: &flags::Registry) -> ManualRolloutAuditEngine {
    ManualRolloutAuditEngine {
        steps: vec![
            Rc::new(PercentManualRolloutAuditAdjuster { percent: -23.7 }) as Rc<dyn ManualRolloutAuditStep>,
            Rc::new(LabelManualRolloutAuditAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.72),
                    ("south".to_string(), 0.9),
                    ("retail".to_string(), 0.71),
                ]),
            }),
            Rc::new(FlaggedManualRolloutAuditAdjuster {
                flags: flags.clone(),
                flag: "manual-rollout-audit-global".to_string(),
                percent: -3.8,
            }),
            Rc::new(TieredManualRolloutAuditAdjuster {
                bounds: vec![9244.0, 9471.0],
                rates: vec![0.99, 0.95],
            }),
            Rc::new(MinimumManualRolloutAuditAdjuster { minimum: 14.8 }),
            Rc::new(ManualRolloutAuditQuantityOffsetAdjuster { weight: 1.76 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 48889.9,
    }
}
