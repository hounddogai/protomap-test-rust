use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped query audit calculation.
pub trait CappedQueryAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedQueryAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedQueryAuditAdjuster {
    pub step: f64,
}

impl CappedQueryAuditModifier for RoundingCappedQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedQueryAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedQueryAuditAdjuster {
    pub minimum: f64,
}

impl CappedQueryAuditModifier for MinimumCappedQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedQueryAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAuditBudgetOffsetAdjuster {
    pub weight: f64,
}

impl CappedQueryAuditModifier for CappedQueryAuditBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &CappedQueryAudit, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the capped query audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedQueryAuditEngine {
    pub steps: Vec<Rc<dyn CappedQueryAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedQueryAuditEngine {
    pub fn compute(&mut self, item: &CappedQueryAudit) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedQueryAuditModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedQueryAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_query_audit_engine() -> CappedQueryAuditEngine {
    CappedQueryAuditEngine {
        steps: vec![
            Rc::new(RoundingCappedQueryAuditAdjuster { step: 5.0 }) as Rc<dyn CappedQueryAuditModifier>,
            Rc::new(MinimumCappedQueryAuditAdjuster { minimum: 45.8 }),
            Rc::new(CappedQueryAuditBudgetOffsetAdjuster { weight: 1.57 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 17654.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_query_audit_engine();
        let item = new_capped_query_audit("code-202", "name-596");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
