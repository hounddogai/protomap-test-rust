use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling policy audit calculation.
pub trait RollingPolicyAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPolicyAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPolicyAuditAdjuster {
    pub cap: f64,
}

impl RollingPolicyAuditModifier for CapRollingPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPolicyAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPolicyAuditAdjuster {
    pub step: f64,
}

impl RollingPolicyAuditModifier for RoundingRollingPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPolicyAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPolicyAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPolicyAuditModifier for TieredRollingPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPolicyAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingPolicyAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPolicyAuditModifier for TitleRollingPolicyAuditAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingPolicyAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyAuditEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl RollingPolicyAuditModifier for RollingPolicyAuditEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPolicyAudit, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the rolling policy audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPolicyAuditEngine {
    pub steps: Vec<Rc<dyn RollingPolicyAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPolicyAuditEngine {
    pub fn run(&mut self, item: &RollingPolicyAudit) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollingPolicyAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_policy_audit_engine() -> RollingPolicyAuditEngine {
    RollingPolicyAuditEngine {
        steps: vec![
            Rc::new(CapRollingPolicyAuditAdjuster { cap: 49920.0 }) as Rc<dyn RollingPolicyAuditModifier>,
            Rc::new(RoundingRollingPolicyAuditAdjuster { step: 0.5 }),
            Rc::new(TieredRollingPolicyAuditAdjuster {
                bounds: vec![762.0, 5545.0, 6405.0, 9813.0],
                rates: vec![0.83, 1.04, 1.05, 1.0],
            }),
            Rc::new(TitleRollingPolicyAuditAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.86),
                    ("central".to_string(), 0.98),
                    ("south".to_string(), 0.87),
                ]),
            }),
            Rc::new(RollingPolicyAuditEndsAtOffsetAdjuster { weight: 0.69 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 90892.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_policy_audit_engine();
        let item = new_rolling_policy_audit("code-651", "title-459");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
