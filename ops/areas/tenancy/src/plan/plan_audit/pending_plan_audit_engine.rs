use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending plan audit calculation.
pub trait PendingPlanAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPlanAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePendingPlanAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPlanAuditStep for ScopePendingPlanAuditAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PendingPlanAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingPlanAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingPlanAuditStep for TieredPendingPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingPlanAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingPlanAuditAdjuster {
    pub percent: f64,
}

impl PendingPlanAuditStep for PercentPendingPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingPlanAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPlanAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPlanAuditStep for FlaggedPendingPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPlanAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingPlanAuditAdjuster {
    pub minimum: f64,
}

impl PendingPlanAuditStep for MinimumPendingPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingPlanAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending plan audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPlanAuditEngine {
    pub steps: Vec<Rc<dyn PendingPlanAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPlanAuditEngine {
    pub fn evaluate(&mut self, item: &PendingPlanAudit) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingPlanAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_plan_audit_engine(flags: &flags::Registry) -> PendingPlanAuditEngine {
    PendingPlanAuditEngine {
        steps: vec![
            Rc::new(ScopePendingPlanAuditAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.22), ("south".to_string(), 1.11)]),
            }) as Rc<dyn PendingPlanAuditStep>,
            Rc::new(TieredPendingPlanAuditAdjuster {
                bounds: vec![3477.0, 5507.0, 7845.0],
                rates: vec![0.86, 1.13, 0.97],
            }),
            Rc::new(PercentPendingPlanAuditAdjuster { percent: -8.4 }),
            Rc::new(FlaggedPendingPlanAuditAdjuster {
                flags: flags.clone(),
                flag: "pending-plan-audit-batched".to_string(),
                percent: 0.8,
            }),
            Rc::new(MinimumPendingPlanAuditAdjuster { minimum: 25.1 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 34845.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_plan_audit_engine(&flags::new_registry());
        let item = new_pending_plan_audit("code-403", "title-46");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
