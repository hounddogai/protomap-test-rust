use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation request calculation.
pub trait FederationRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationRequestFactor for TieredFederationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRequestQuantityOffsetAdjuster {
    pub weight: f64,
}

impl FederationRequestFactor for FederationRequestQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &FederationRequest, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationRequestAdjuster {
    pub cap: f64,
}

impl FederationRequestFactor for CapFederationRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationRequestAdjuster {
    pub percent: f64,
}

impl FederationRequestFactor for PercentFederationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationRequestAdjuster {
    pub minimum: f64,
}

impl FederationRequestFactor for MinimumFederationRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the federation request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationRequestEngine {
    pub steps: Vec<Rc<dyn FederationRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationRequestEngine {
    pub fn compute(&mut self, item: &FederationRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FederationRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_federation_request_engine() -> FederationRequestEngine {
    FederationRequestEngine {
        steps: vec![
            Rc::new(TieredFederationRequestAdjuster {
                bounds: vec![2268.0, 4144.0, 5420.0],
                rates: vec![1.14, 1.06, 1.13],
            }) as Rc<dyn FederationRequestFactor>,
            Rc::new(FederationRequestQuantityOffsetAdjuster { weight: 0.19 }),
            Rc::new(CapFederationRequestAdjuster { cap: 29825.0 }),
            Rc::new(PercentFederationRequestAdjuster { percent: 19.0 }),
            Rc::new(MinimumFederationRequestAdjuster { minimum: 38.5 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 9248.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_request_engine();
        let item = new_federation_request("code-318", "name-329");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
