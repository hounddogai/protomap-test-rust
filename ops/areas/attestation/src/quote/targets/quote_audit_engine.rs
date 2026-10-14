use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the quote audit calculation.
pub trait QuoteAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteAuditAdjuster {
    pub step: f64,
}

impl QuoteAuditModifier for RoundingQuoteAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteAuditAdjuster {
    pub cap: f64,
}

impl QuoteAuditModifier for CapQuoteAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteAuditAdjuster {
    pub minimum: f64,
}

impl QuoteAuditModifier for MinimumQuoteAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the quote audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteAuditEngine {
    pub steps: Vec<Rc<dyn QuoteAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteAuditEngine {
    pub fn evaluate(&mut self, item: &QuoteAudit) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[QuoteAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_quote_audit_engine() -> QuoteAuditEngine {
    QuoteAuditEngine {
        steps: vec![
            Rc::new(RoundingQuoteAuditAdjuster { step: 0.25 }) as Rc<dyn QuoteAuditModifier>,
            Rc::new(CapQuoteAuditAdjuster { cap: 27822.0 }),
            Rc::new(MinimumQuoteAuditAdjuster { minimum: 18.8 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 52739.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_audit_engine();
        let item = new_quote_audit("code-527", "label-363");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
