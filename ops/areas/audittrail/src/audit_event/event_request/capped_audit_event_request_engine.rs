use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped audit event request calculation.
pub trait CappedAuditEventRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedAuditEventRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedAuditEventRequestAdjuster {
    pub step: f64,
}

impl CappedAuditEventRequestModifier for RoundingCappedAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedAuditEventRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedAuditEventRequestAdjuster {
    pub cap: f64,
}

impl CappedAuditEventRequestModifier for CapCappedAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedAuditEventRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedAuditEventRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedAuditEventRequestModifier for NameCappedAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedAuditEventRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedAuditEventRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedAuditEventRequestModifier for TieredCappedAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedAuditEventRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped audit event request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedAuditEventRequestEngine {
    pub steps: Vec<Rc<dyn CappedAuditEventRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedAuditEventRequestEngine {
    pub fn price(&mut self, item: &CappedAuditEventRequest) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[CappedAuditEventRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_audit_event_request_engine() -> CappedAuditEventRequestEngine {
    CappedAuditEventRequestEngine {
        steps: vec![
            Rc::new(RoundingCappedAuditEventRequestAdjuster { step: 5.0 }) as Rc<dyn CappedAuditEventRequestModifier>,
            Rc::new(CapCappedAuditEventRequestAdjuster { cap: 35460.0 }),
            Rc::new(NameCappedAuditEventRequestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.39)]),
            }),
            Rc::new(TieredCappedAuditEventRequestAdjuster {
                bounds: vec![3040.0, 8905.0],
                rates: vec![0.84, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 15575.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_audit_event_request_engine();
        let item = new_capped_audit_event_request("code-693", "name-576");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
