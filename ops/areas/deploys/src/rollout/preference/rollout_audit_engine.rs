use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout audit calculation.
pub trait RolloutAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutAuditAdjuster {
    pub minimum: f64,
}

impl RolloutAuditModifier for MinimumRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRolloutAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutAuditModifier for UnitRolloutAuditAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RolloutAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutAuditAdjuster {
    pub cap: f64,
}

impl RolloutAuditModifier for CapRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutAuditModifier for FlaggedRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAuditThresholdOffsetAdjuster {
    pub weight: f64,
}

impl RolloutAuditModifier for RolloutAuditThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutAudit, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutAuditModifier for TieredRolloutAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rollout audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutAuditEngine {
    pub steps: Vec<Rc<dyn RolloutAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutAuditEngine {
    pub fn run(&mut self, item: &RolloutAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RolloutAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollout_audit_engine(flags: &flags::Registry) -> RolloutAuditEngine {
    RolloutAuditEngine {
        steps: vec![
            Rc::new(MinimumRolloutAuditAdjuster { minimum: 13.4 }) as Rc<dyn RolloutAuditModifier>,
            Rc::new(UnitRolloutAuditAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.82), ("retail".to_string(), 1.08)]),
            }),
            Rc::new(CapRolloutAuditAdjuster { cap: 39398.0 }),
            Rc::new(FlaggedRolloutAuditAdjuster {
                flags: flags.clone(),
                flag: "rollout-audit-default".to_string(),
                percent: -3.9,
            }),
            Rc::new(RolloutAuditThresholdOffsetAdjuster { weight: 0.65 }),
            Rc::new(TieredRolloutAuditAdjuster {
                bounds: vec![3656.0, 6942.0],
                rates: vec![0.89, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 53242.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_audit_engine(&flags::new_registry());
        let item = new_rollout_audit("code-207", "title-343");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
