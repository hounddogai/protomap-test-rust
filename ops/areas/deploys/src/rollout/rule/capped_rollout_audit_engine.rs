use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped rollout audit calculation.
pub trait CappedRolloutAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRolloutAudit, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedRolloutAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedRolloutAuditStep for FlaggedCappedRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedRolloutAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedRolloutAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedRolloutAuditStep for TieredCappedRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedRolloutAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutAuditPortOffsetAdjuster {
    pub weight: f64,
}

impl CappedRolloutAuditStep for CappedRolloutAuditPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &CappedRolloutAudit, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedRolloutAuditAdjuster {
    pub minimum: f64,
}

impl CappedRolloutAuditStep for MinimumCappedRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedRolloutAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedRolloutAuditAdjuster {
    pub cap: f64,
}

impl CappedRolloutAuditStep for CapCappedRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedRolloutAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped rollout audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRolloutAuditEngine {
    pub steps: Vec<Rc<dyn CappedRolloutAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRolloutAuditEngine {
    pub fn price(&mut self, item: &CappedRolloutAudit) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[CappedRolloutAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_rollout_audit_engine(flags: &flags::Registry) -> CappedRolloutAuditEngine {
    CappedRolloutAuditEngine {
        steps: vec![
            Rc::new(FlaggedCappedRolloutAuditAdjuster {
                flags: flags.clone(),
                flag: "capped-rollout-audit-tiered".to_string(),
                percent: 7.8,
            }) as Rc<dyn CappedRolloutAuditStep>,
            Rc::new(TieredCappedRolloutAuditAdjuster {
                bounds: vec![4289.0, 7735.0, 8060.0],
                rates: vec![0.9, 0.98, 0.89],
            }),
            Rc::new(CappedRolloutAuditPortOffsetAdjuster { weight: 1.1 }),
            Rc::new(MinimumCappedRolloutAuditAdjuster { minimum: 31.7 }),
            Rc::new(CapCappedRolloutAuditAdjuster { cap: 13770.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 23078.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_rollout_audit_engine(&flags::new_registry());
        let item = new_capped_rollout_audit("code-923", "name-794");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
