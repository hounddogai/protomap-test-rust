use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event milestone calculation.
pub trait AuditEventMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventMilestoneAdjuster {
    pub step: f64,
}

impl AuditEventMilestoneModifier for RoundingAuditEventMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventMilestoneAdjuster {
    pub minimum: f64,
}

impl AuditEventMilestoneModifier for MinimumAuditEventMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventMilestoneAdjuster {
    pub cap: f64,
}

impl AuditEventMilestoneModifier for CapAuditEventMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAuditEventMilestoneAdjuster {
    pub percent: f64,
}

impl AuditEventMilestoneModifier for PercentAuditEventMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AuditEventMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventMilestoneModifier for TieredAuditEventMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the audit event milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventMilestoneEngine {
    pub steps: Vec<Rc<dyn AuditEventMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventMilestoneEngine {
    pub fn evaluate(&mut self, item: &AuditEventMilestone) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventMilestoneModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_audit_event_milestone_engine() -> AuditEventMilestoneEngine {
    AuditEventMilestoneEngine {
        steps: vec![
            Rc::new(RoundingAuditEventMilestoneAdjuster { step: 0.5 }) as Rc<dyn AuditEventMilestoneModifier>,
            Rc::new(MinimumAuditEventMilestoneAdjuster { minimum: 48.6 }),
            Rc::new(CapAuditEventMilestoneAdjuster { cap: 27626.0 }),
            Rc::new(PercentAuditEventMilestoneAdjuster { percent: -14.1 }),
            Rc::new(TieredAuditEventMilestoneAdjuster {
                bounds: vec![6782.0, 9439.0, 9912.0],
                rates: vec![1.02, 1.06, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 71949.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_milestone_engine();
        let item = new_audit_event_milestone("code-364", "name-122");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
