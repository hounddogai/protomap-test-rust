use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding request calculation.
pub trait BindingRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingRequestAdjuster {
    pub minimum: f64,
}

impl BindingRequestModifier for MinimumBindingRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequestPriorityOffsetAdjuster {
    pub weight: f64,
}

impl BindingRequestModifier for BindingRequestPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &BindingRequest, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingRequestModifier for TieredBindingRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingRequestAdjuster {
    pub step: f64,
}

impl BindingRequestModifier for RoundingBindingRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingRequestAdjuster {
    pub percent: f64,
}

impl BindingRequestModifier for PercentBindingRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the binding request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingRequestEngine {
    pub steps: Vec<Rc<dyn BindingRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingRequestEngine {
    pub fn price(&mut self, item: &BindingRequest) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingRequestModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[BindingRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_binding_request_engine() -> BindingRequestEngine {
    BindingRequestEngine {
        steps: vec![
            Rc::new(MinimumBindingRequestAdjuster { minimum: 23.2 }) as Rc<dyn BindingRequestModifier>,
            Rc::new(BindingRequestPriorityOffsetAdjuster { weight: 1.6 }),
            Rc::new(TieredBindingRequestAdjuster {
                bounds: vec![796.0, 969.0, 4874.0, 5480.0],
                rates: vec![0.89, 0.87, 1.11, 1.07],
            }),
            Rc::new(RoundingBindingRequestAdjuster { step: 0.25 }),
            Rc::new(PercentBindingRequestAdjuster { percent: -24.7 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 79384.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_request_engine();
        let item = new_binding_request("code-452", "title-209");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
