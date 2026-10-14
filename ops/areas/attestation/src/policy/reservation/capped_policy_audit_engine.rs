use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped policy audit calculation.
pub trait CappedPolicyAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPolicyAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedPolicyAuditAdjuster {
    pub cap: f64,
}

impl CappedPolicyAuditFactor for CapCappedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedPolicyAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedPolicyAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedPolicyAuditFactor for TieredCappedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedPolicyAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAuditCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl CappedPolicyAuditFactor for CappedPolicyAuditCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &CappedPolicyAudit, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceCappedPolicyAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedPolicyAuditFactor for SourceCappedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &CappedPolicyAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedPolicyAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedPolicyAuditFactor for FlaggedCappedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedPolicyAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedPolicyAuditAdjuster {
    pub percent: f64,
}

impl CappedPolicyAuditFactor for PercentCappedPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedPolicyAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped policy audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPolicyAuditEngine {
    pub steps: Vec<Rc<dyn CappedPolicyAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPolicyAuditEngine {
    pub fn compute(&mut self, item: &CappedPolicyAudit) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedPolicyAuditFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedPolicyAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_policy_audit_engine(flags: &flags::Registry) -> CappedPolicyAuditEngine {
    CappedPolicyAuditEngine {
        steps: vec![
            Rc::new(CapCappedPolicyAuditAdjuster { cap: 19572.0 }) as Rc<dyn CappedPolicyAuditFactor>,
            Rc::new(TieredCappedPolicyAuditAdjuster {
                bounds: vec![5271.0, 9531.0],
                rates: vec![1.06, 0.92],
            }),
            Rc::new(CappedPolicyAuditCreatedAtOffsetAdjuster { weight: 0.75 }),
            Rc::new(SourceCappedPolicyAuditAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.93),
                    ("west".to_string(), 1.31),
                    ("central".to_string(), 0.81),
                ]),
            }),
            Rc::new(FlaggedCappedPolicyAuditAdjuster {
                flags: flags.clone(),
                flag: "capped-policy-audit-legacy".to_string(),
                percent: 4.6,
            }),
            Rc::new(PercentCappedPolicyAuditAdjuster { percent: 19.4 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 20605.4,
    }
}
