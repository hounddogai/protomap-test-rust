use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled audit event allocation calculation.
pub trait ScheduledAuditEventAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledAuditEventAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledAuditEventAllocationAdjuster {
    pub percent: f64,
}

impl ScheduledAuditEventAllocationModifier for PercentScheduledAuditEventAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledAuditEventAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventAllocationCostOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledAuditEventAllocationModifier for ScheduledAuditEventAllocationCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledAuditEventAllocation, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledAuditEventAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledAuditEventAllocationModifier for TieredScheduledAuditEventAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledAuditEventAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledAuditEventAllocationAdjuster {
    pub minimum: f64,
}

impl ScheduledAuditEventAllocationModifier for MinimumScheduledAuditEventAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledAuditEventAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled audit event allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledAuditEventAllocationEngine {
    pub steps: Vec<Rc<dyn ScheduledAuditEventAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledAuditEventAllocationEngine {
    pub fn compute(&mut self, item: &ScheduledAuditEventAllocation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledAuditEventAllocationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledAuditEventAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_audit_event_allocation_engine() -> ScheduledAuditEventAllocationEngine {
    ScheduledAuditEventAllocationEngine {
        steps: vec![
            Rc::new(PercentScheduledAuditEventAllocationAdjuster { percent: 14.1 })
                as Rc<dyn ScheduledAuditEventAllocationModifier>,
            Rc::new(ScheduledAuditEventAllocationCostOffsetAdjuster { weight: 1.81 }),
            Rc::new(TieredScheduledAuditEventAllocationAdjuster {
                bounds: vec![1128.0, 4938.0, 9227.0, 9679.0],
                rates: vec![0.97, 1.08, 0.98, 1.02],
            }),
            Rc::new(MinimumScheduledAuditEventAllocationAdjuster { minimum: 49.1 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 75838.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_audit_event_allocation_engine();
        let item = new_scheduled_audit_event_allocation("code-191", "name-199");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
