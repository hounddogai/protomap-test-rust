use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the grant audit calculation.
pub trait GrantAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantAuditAdjuster {
    pub minimum: f64,
}

impl GrantAuditFactor for MinimumGrantAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantAuditAdjuster {
    pub percent: f64,
}

impl GrantAuditFactor for PercentGrantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantAuditFactor for FlaggedGrantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantAuditAdjuster {
    pub cap: f64,
}

impl GrantAuditFactor for CapGrantAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAuditQuantityOffsetAdjuster {
    pub weight: f64,
}

impl GrantAuditFactor for GrantAuditQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &GrantAudit, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantAuditFactor for TieredGrantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the grant audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantAuditEngine {
    pub steps: Vec<Rc<dyn GrantAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantAuditEngine {
    pub fn evaluate(&mut self, item: &GrantAudit) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantAuditFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_grant_audit_engine(flags: &flags::Registry) -> GrantAuditEngine {
    GrantAuditEngine {
        steps: vec![
            Rc::new(MinimumGrantAuditAdjuster { minimum: 34.3 }) as Rc<dyn GrantAuditFactor>,
            Rc::new(PercentGrantAuditAdjuster { percent: 20.5 }),
            Rc::new(FlaggedGrantAuditAdjuster {
                flags: flags.clone(),
                flag: "grant-audit-primary".to_string(),
                percent: 4.4,
            }),
            Rc::new(CapGrantAuditAdjuster { cap: 10043.0 }),
            Rc::new(GrantAuditQuantityOffsetAdjuster { weight: 1.36 }),
            Rc::new(TieredGrantAuditAdjuster {
                bounds: vec![5321.0, 9750.0],
                rates: vec![0.97, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 88731.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_audit_engine(&flags::new_registry());
        let item = new_grant_audit("code-268", "title-653");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
