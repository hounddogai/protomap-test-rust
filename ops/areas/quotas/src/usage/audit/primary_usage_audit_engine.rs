use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary usage audit calculation.
pub trait PrimaryUsageAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryUsageAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryUsageAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryUsageAuditAdjuster for TieredPrimaryUsageAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryUsageAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAuditAmountOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryUsageAuditAdjuster for PrimaryUsageAuditAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryUsageAudit, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryUsageAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryUsageAuditAdjuster for SourcePrimaryUsageAuditAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryUsageAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryUsageAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryUsageAuditAdjuster for FlaggedPrimaryUsageAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryUsageAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryUsageAuditAdjuster {
    pub minimum: f64,
}

impl PrimaryUsageAuditAdjuster for MinimumPrimaryUsageAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryUsageAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary usage audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryUsageAuditEngine {
    pub steps: Vec<Rc<dyn PrimaryUsageAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryUsageAuditEngine {
    pub fn evaluate(&mut self, item: &PrimaryUsageAudit) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryUsageAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryUsageAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_usage_audit_engine(flags: &flags::Registry) -> PrimaryUsageAuditEngine {
    PrimaryUsageAuditEngine {
        steps: vec![
            Rc::new(TieredPrimaryUsageAuditAdjuster {
                bounds: vec![1303.0, 6261.0, 9686.0],
                rates: vec![1.12, 0.84, 0.97],
            }) as Rc<dyn PrimaryUsageAuditAdjuster>,
            Rc::new(PrimaryUsageAuditAmountOffsetAdjuster { weight: 0.46 }),
            Rc::new(SourcePrimaryUsageAuditAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.96), ("south".to_string(), 1.15)]),
            }),
            Rc::new(FlaggedPrimaryUsageAuditAdjuster {
                flags: flags.clone(),
                flag: "primary-usage-audit-manual".to_string(),
                percent: 1.3,
            }),
            Rc::new(MinimumPrimaryUsageAuditAdjuster { minimum: 49.1 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 14494.8,
    }
}
