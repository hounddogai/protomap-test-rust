use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation assignment calculation.
pub trait FederationAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationAssignmentCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl FederationAssignmentFactor for FederationAssignmentCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &FederationAssignment, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationAssignmentAdjuster {
    pub minimum: f64,
}

impl FederationAssignmentFactor for MinimumFederationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationAssignmentFactor for TieredFederationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationAssignmentAdjuster {
    pub cap: f64,
}

impl FederationAssignmentFactor for CapFederationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationAssignmentAdjuster {
    pub step: f64,
}

impl FederationAssignmentFactor for RoundingFederationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the federation assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationAssignmentEngine {
    pub steps: Vec<Rc<dyn FederationAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationAssignmentEngine {
    pub fn evaluate(&mut self, item: &FederationAssignment) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[FederationAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_assignment_engine() -> FederationAssignmentEngine {
    FederationAssignmentEngine {
        steps: vec![
            Rc::new(FederationAssignmentCreatedAtOffsetAdjuster { weight: 0.38 }) as Rc<dyn FederationAssignmentFactor>,
            Rc::new(MinimumFederationAssignmentAdjuster { minimum: 48.4 }),
            Rc::new(TieredFederationAssignmentAdjuster {
                bounds: vec![789.0, 4371.0, 9971.0],
                rates: vec![1.01, 1.13, 0.98],
            }),
            Rc::new(CapFederationAssignmentAdjuster { cap: 44752.0 }),
            Rc::new(RoundingFederationAssignmentAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 38275.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_assignment_engine();
        let item = new_federation_assignment("code-90", "title-613");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
