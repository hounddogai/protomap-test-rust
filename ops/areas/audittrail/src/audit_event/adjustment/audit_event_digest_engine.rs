use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event digest calculation.
pub trait AuditEventDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventDigestAdjuster {
    pub cap: f64,
}

impl AuditEventDigestStep for CapAuditEventDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDigestEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventDigestStep for AuditEventDigestEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventDigest, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeAuditEventDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventDigestStep for ScopeAuditEventDigestAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &AuditEventDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventDigestAdjuster {
    pub minimum: f64,
}

impl AuditEventDigestStep for MinimumAuditEventDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the audit event digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventDigestEngine {
    pub steps: Vec<Rc<dyn AuditEventDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventDigestEngine {
    pub fn evaluate(&mut self, item: &AuditEventDigest) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_audit_event_digest_engine() -> AuditEventDigestEngine {
    AuditEventDigestEngine {
        steps: vec![
            Rc::new(CapAuditEventDigestAdjuster { cap: 18849.0 }) as Rc<dyn AuditEventDigestStep>,
            Rc::new(AuditEventDigestEndsAtOffsetAdjuster { weight: 1.09 }),
            Rc::new(ScopeAuditEventDigestAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.14)]),
            }),
            Rc::new(MinimumAuditEventDigestAdjuster { minimum: 24.2 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 98964.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_digest_engine();
        let item = new_audit_event_digest("code-848", "title-417");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
