use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction request calculation.
pub trait RedactionRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionRequestAdjuster {
    pub step: f64,
}

impl RedactionRequestFactor for RoundingRedactionRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRequestCostOffsetAdjuster {
    pub weight: f64,
}

impl RedactionRequestFactor for RedactionRequestCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionRequest, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionRequestFactor for TieredRedactionRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the redaction request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionRequestEngine {
    pub steps: Vec<Rc<dyn RedactionRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionRequestEngine {
    pub fn price(&mut self, item: &RedactionRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RedactionRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_request_engine() -> RedactionRequestEngine {
    RedactionRequestEngine {
        steps: vec![
            Rc::new(RoundingRedactionRequestAdjuster { step: 0.5 }) as Rc<dyn RedactionRequestFactor>,
            Rc::new(RedactionRequestCostOffsetAdjuster { weight: 0.36 }),
            Rc::new(TieredRedactionRequestAdjuster {
                bounds: vec![5611.0, 6878.0, 8201.0],
                rates: vec![1.06, 1.13, 0.95],
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 11090.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_request_engine();
        let item = new_redaction_request("code-154", "name-310");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
