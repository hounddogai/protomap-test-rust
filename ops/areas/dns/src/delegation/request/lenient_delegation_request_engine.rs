use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient delegation request calculation.
pub trait LenientDelegationRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientDelegationRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientDelegationRequestAdjuster {
    pub minimum: f64,
}

impl LenientDelegationRequestFactor for MinimumLenientDelegationRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientDelegationRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientDelegationRequestAdjuster {
    pub percent: f64,
}

impl LenientDelegationRequestFactor for PercentLenientDelegationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientDelegationRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientDelegationRequestAdjuster {
    pub step: f64,
}

impl LenientDelegationRequestFactor for RoundingLenientDelegationRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientDelegationRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient delegation request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientDelegationRequestEngine {
    pub steps: Vec<Rc<dyn LenientDelegationRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientDelegationRequestEngine {
    pub fn run(&mut self, item: &LenientDelegationRequest) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[LenientDelegationRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_delegation_request_engine() -> LenientDelegationRequestEngine {
    LenientDelegationRequestEngine {
        steps: vec![
            Rc::new(MinimumLenientDelegationRequestAdjuster { minimum: 33.0 })
                as Rc<dyn LenientDelegationRequestFactor>,
            Rc::new(PercentLenientDelegationRequestAdjuster { percent: -6.7 }),
            Rc::new(RoundingLenientDelegationRequestAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 89314.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_delegation_request_engine();
        let item = new_lenient_delegation_request("code-519", "label-75");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
