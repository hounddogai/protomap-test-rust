use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event exception calculation.
pub trait AuditEventExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventExceptionCapacityOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventExceptionStep for AuditEventExceptionCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventException, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventExceptionAdjuster {
    pub minimum: f64,
}

impl AuditEventExceptionStep for MinimumAuditEventExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventExceptionStep for TieredAuditEventExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the audit event exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventExceptionEngine {
    pub steps: Vec<Rc<dyn AuditEventExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventExceptionEngine {
    pub fn run(&mut self, item: &AuditEventException) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_audit_event_exception_engine() -> AuditEventExceptionEngine {
    AuditEventExceptionEngine {
        steps: vec![
            Rc::new(AuditEventExceptionCapacityOffsetAdjuster { weight: 0.07 }) as Rc<dyn AuditEventExceptionStep>,
            Rc::new(MinimumAuditEventExceptionAdjuster { minimum: 12.4 }),
            Rc::new(TieredAuditEventExceptionAdjuster {
                bounds: vec![3812.0, 6347.0, 6508.0],
                rates: vec![1.16, 1.15, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 90767.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_exception_engine();
        let item = new_audit_event_exception("code-532", "title-366");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
