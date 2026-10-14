use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering group calculation.
pub trait PeeringGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringGroupAdjuster {
    pub minimum: f64,
}

impl PeeringGroupFactor for MinimumPeeringGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringGroupAdjuster {
    pub step: f64,
}

impl PeeringGroupFactor for RoundingPeeringGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringGroupPriorityOffsetAdjuster {
    pub weight: f64,
}

impl PeeringGroupFactor for PeeringGroupPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringGroup, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the peering group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringGroupEngine {
    pub steps: Vec<Rc<dyn PeeringGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringGroupEngine {
    pub fn evaluate(&mut self, item: &PeeringGroup) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_group_engine() -> PeeringGroupEngine {
    PeeringGroupEngine {
        steps: vec![
            Rc::new(MinimumPeeringGroupAdjuster { minimum: 8.0 }) as Rc<dyn PeeringGroupFactor>,
            Rc::new(RoundingPeeringGroupAdjuster { step: 0.25 }),
            Rc::new(PeeringGroupPriorityOffsetAdjuster { weight: 1.8 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 62488.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_group_engine();
        let item = new_peering_group("code-282", "title-481");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
