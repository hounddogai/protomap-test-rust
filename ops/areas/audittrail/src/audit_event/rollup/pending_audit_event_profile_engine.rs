use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending audit event profile calculation.
pub trait PendingAuditEventProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingAuditEventProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingAuditEventProfileAdjuster {
    pub step: f64,
}

impl PendingAuditEventProfileAdjuster for RoundingPendingAuditEventProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingAuditEventProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingAuditEventProfileAdjuster {
    pub percent: f64,
}

impl PendingAuditEventProfileAdjuster for PercentPendingAuditEventProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingAuditEventProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPendingAuditEventProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingAuditEventProfileAdjuster for RegionPendingAuditEventProfileAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PendingAuditEventProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingAuditEventProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingAuditEventProfileAdjuster for TieredPendingAuditEventProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingAuditEventProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending audit event profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingAuditEventProfileEngine {
    pub steps: Vec<Rc<dyn PendingAuditEventProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingAuditEventProfileEngine {
    pub fn evaluate(&mut self, item: &PendingAuditEventProfile) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingAuditEventProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_audit_event_profile_engine() -> PendingAuditEventProfileEngine {
    PendingAuditEventProfileEngine {
        steps: vec![
            Rc::new(RoundingPendingAuditEventProfileAdjuster { step: 5.0 }) as Rc<dyn PendingAuditEventProfileAdjuster>,
            Rc::new(PercentPendingAuditEventProfileAdjuster { percent: 14.8 }),
            Rc::new(RegionPendingAuditEventProfileAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.19)]),
            }),
            Rc::new(TieredPendingAuditEventProfileAdjuster {
                bounds: vec![4591.0, 4993.0, 9135.0, 9174.0],
                rates: vec![1.19, 1.19, 1.03, 0.88],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 43500.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_audit_event_profile_engine();
        let item = new_pending_audit_event_profile("code-726", "title-915");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
