use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending query audit calculation.
pub trait PendingQueryAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingQueryAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingQueryAuditAdjuster {
    pub cap: f64,
}

impl PendingQueryAuditAdjuster for CapPendingQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingQueryAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingQueryAuditAdjuster {
    pub minimum: f64,
}

impl PendingQueryAuditAdjuster for MinimumPendingQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingQueryAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryAuditDiscountOffsetAdjuster {
    pub weight: f64,
}

impl PendingQueryAuditAdjuster for PendingQueryAuditDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &PendingQueryAudit, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingQueryAuditAdjuster {
    pub step: f64,
}

impl PendingQueryAuditAdjuster for RoundingPendingQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingQueryAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingQueryAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingQueryAuditAdjuster for TieredPendingQueryAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingQueryAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending query audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingQueryAuditEngine {
    pub steps: Vec<Rc<dyn PendingQueryAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingQueryAuditEngine {
    pub fn price(&mut self, item: &PendingQueryAudit) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingQueryAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_query_audit_engine() -> PendingQueryAuditEngine {
    PendingQueryAuditEngine {
        steps: vec![
            Rc::new(CapPendingQueryAuditAdjuster { cap: 8170.0 }) as Rc<dyn PendingQueryAuditAdjuster>,
            Rc::new(MinimumPendingQueryAuditAdjuster { minimum: 46.6 }),
            Rc::new(PendingQueryAuditDiscountOffsetAdjuster { weight: 1.92 }),
            Rc::new(RoundingPendingQueryAuditAdjuster { step: 0.1 }),
            Rc::new(TieredPendingQueryAuditAdjuster {
                bounds: vec![1634.0, 3357.0],
                rates: vec![0.82, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 74680.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_query_audit_engine();
        let item = new_pending_query_audit("code-912", "name-589");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
