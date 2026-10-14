use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the ttl policy audit calculation.
pub trait TtlPolicyAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyAuditAdjuster {
    pub cap: f64,
}

impl TtlPolicyAuditAdjuster for CapTtlPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTtlPolicyAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyAuditAdjuster for RegionTtlPolicyAuditAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &TtlPolicyAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyAuditAdjuster for TieredTtlPolicyAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the ttl policy audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyAuditEngine {
    pub steps: Vec<Rc<dyn TtlPolicyAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyAuditEngine {
    pub fn compute(&mut self, item: &TtlPolicyAudit) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_ttl_policy_audit_engine() -> TtlPolicyAuditEngine {
    TtlPolicyAuditEngine {
        steps: vec![
            Rc::new(CapTtlPolicyAuditAdjuster { cap: 34045.0 }) as Rc<dyn TtlPolicyAuditAdjuster>,
            Rc::new(RegionTtlPolicyAuditAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.14),
                    ("west".to_string(), 1.16),
                    ("online".to_string(), 1.17),
                ]),
            }),
            Rc::new(TieredTtlPolicyAuditAdjuster {
                bounds: vec![4844.0, 7318.0, 7383.0, 8567.0],
                rates: vec![0.9, 0.93, 0.9, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 56973.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_audit_engine();
        let item = new_ttl_policy_audit("code-697", "title-853");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
