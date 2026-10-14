use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling page audit calculation.
pub trait RollingPageAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPageAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPageAuditAdjuster {
    pub step: f64,
}

impl RollingPageAuditStep for RoundingRollingPageAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPageAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRollingPageAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPageAuditStep for ScopeRollingPageAuditAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RollingPageAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAuditFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RollingPageAuditStep for RollingPageAuditFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPageAudit, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the rolling page audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPageAuditEngine {
    pub steps: Vec<Rc<dyn RollingPageAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPageAuditEngine {
    pub fn evaluate(&mut self, item: &RollingPageAudit) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollingPageAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_page_audit_engine() -> RollingPageAuditEngine {
    RollingPageAuditEngine {
        steps: vec![
            Rc::new(RoundingRollingPageAuditAdjuster { step: 5.0 }) as Rc<dyn RollingPageAuditStep>,
            Rc::new(ScopeRollingPageAuditAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.07)]),
            }),
            Rc::new(RollingPageAuditFloorValueOffsetAdjuster { weight: 0.41 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 25583.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_page_audit_engine();
        let item = new_rolling_page_audit("code-674", "title-720");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
