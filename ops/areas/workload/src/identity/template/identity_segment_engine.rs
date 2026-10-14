use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity segment calculation.
pub trait IdentitySegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentitySegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySegmentFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl IdentitySegmentFactor for IdentitySegmentFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &IdentitySegment, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentitySegmentAdjuster {
    pub step: f64,
}

impl IdentitySegmentFactor for RoundingIdentitySegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentitySegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentitySegmentAdjuster {
    pub minimum: f64,
}

impl IdentitySegmentFactor for MinimumIdentitySegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentitySegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the identity segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentitySegmentEngine {
    pub steps: Vec<Rc<dyn IdentitySegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentitySegmentEngine {
    pub fn evaluate(&mut self, item: &IdentitySegment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentitySegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentitySegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_segment_engine() -> IdentitySegmentEngine {
    IdentitySegmentEngine {
        steps: vec![
            Rc::new(IdentitySegmentFloorValueOffsetAdjuster { weight: 0.83 }) as Rc<dyn IdentitySegmentFactor>,
            Rc::new(RoundingIdentitySegmentAdjuster { step: 0.05 }),
            Rc::new(MinimumIdentitySegmentAdjuster { minimum: 5.2 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 35021.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_segment_engine();
        let item = new_identity_segment("code-643", "name-225");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
