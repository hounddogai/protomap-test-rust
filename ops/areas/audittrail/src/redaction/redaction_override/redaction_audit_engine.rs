use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction audit calculation.
pub trait RedactionAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionAuditAdjuster {
    pub step: f64,
}

impl RedactionAuditModifier for RoundingRedactionAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRedactionAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionAuditModifier for TitleRedactionAuditAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RedactionAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionAuditModifier for TieredRedactionAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the redaction audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionAuditEngine {
    pub steps: Vec<Rc<dyn RedactionAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionAuditEngine {
    pub fn price(&mut self, item: &RedactionAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RedactionAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_audit_engine() -> RedactionAuditEngine {
    RedactionAuditEngine {
        steps: vec![
            Rc::new(RoundingRedactionAuditAdjuster { step: 1.0 }) as Rc<dyn RedactionAuditModifier>,
            Rc::new(TitleRedactionAuditAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.78), ("central".to_string(), 1.25)]),
            }),
            Rc::new(TieredRedactionAuditAdjuster {
                bounds: vec![749.0, 3738.0, 7719.0],
                rates: vec![1.16, 0.87, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 24561.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_audit_engine();
        let item = new_redaction_audit("code-956", "title-143");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
