use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling federation request calculation.
pub trait RollingFederationRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingFederationRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingFederationRequestAdjuster {
    pub cap: f64,
}

impl RollingFederationRequestFactor for CapRollingFederationRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingFederationRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationRequestMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RollingFederationRequestFactor for RollingFederationRequestMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RollingFederationRequest, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingFederationRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingFederationRequestFactor for TieredRollingFederationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingFederationRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingFederationRequestAdjuster {
    pub minimum: f64,
}

impl RollingFederationRequestFactor for MinimumRollingFederationRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingFederationRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rolling federation request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingFederationRequestEngine {
    pub steps: Vec<Rc<dyn RollingFederationRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingFederationRequestEngine {
    pub fn evaluate(&mut self, item: &RollingFederationRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingFederationRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_federation_request_engine() -> RollingFederationRequestEngine {
    RollingFederationRequestEngine {
        steps: vec![
            Rc::new(CapRollingFederationRequestAdjuster { cap: 49405.0 }) as Rc<dyn RollingFederationRequestFactor>,
            Rc::new(RollingFederationRequestMaximumOffsetAdjuster { weight: 1.96 }),
            Rc::new(TieredRollingFederationRequestAdjuster {
                bounds: vec![1257.0, 1504.0, 2630.0],
                rates: vec![1.0, 1.0, 1.03],
            }),
            Rc::new(MinimumRollingFederationRequestAdjuster { minimum: 14.9 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 78552.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_federation_request_engine();
        let item = new_rolling_federation_request("code-446", "title-598");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
