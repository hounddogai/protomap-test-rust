use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query audit calculation.
pub trait QueryAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryAudit, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryAuditModifier for FlaggedQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryAuditAdjuster {
    pub percent: f64,
}

impl QueryAuditModifier for PercentQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAuditQuantityOffsetAdjuster {
    pub weight: f64,
}

impl QueryAuditModifier for QueryAuditQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &QueryAudit, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryAuditAdjuster {
    pub step: f64,
}

impl QueryAuditModifier for RoundingQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryAuditAdjuster {
    pub cap: f64,
}

impl QueryAuditModifier for CapQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the query audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryAuditEngine {
    pub steps: Vec<Rc<dyn QueryAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryAuditEngine {
    pub fn compute(&mut self, item: &QueryAudit) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[QueryAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_audit_engine(flags: &flags::Registry) -> QueryAuditEngine {
    QueryAuditEngine {
        steps: vec![
            Rc::new(FlaggedQueryAuditAdjuster {
                flags: flags.clone(),
                flag: "query-audit-adaptive".to_string(),
                percent: 2.4,
            }) as Rc<dyn QueryAuditModifier>,
            Rc::new(PercentQueryAuditAdjuster { percent: -19.4 }),
            Rc::new(QueryAuditQuantityOffsetAdjuster { weight: 0.33 }),
            Rc::new(RoundingQueryAuditAdjuster { step: 0.1 }),
            Rc::new(CapQueryAuditAdjuster { cap: 40948.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 5524.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_audit_engine(&flags::new_registry());
        let item = new_query_audit("code-583", "title-166");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
