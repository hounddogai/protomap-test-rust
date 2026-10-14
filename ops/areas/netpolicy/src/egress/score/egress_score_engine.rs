use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress score calculation.
pub trait EgressScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressScoreAdjuster {
    pub minimum: f64,
}

impl EgressScoreStep for MinimumEgressScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressScoreFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl EgressScoreStep for EgressScoreFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &EgressScore, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressScoreAdjuster {
    pub step: f64,
}

impl EgressScoreStep for RoundingEgressScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressScoreStep for TieredEgressScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the egress score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressScoreEngine {
    pub steps: Vec<Rc<dyn EgressScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressScoreEngine {
    pub fn compute(&mut self, item: &EgressScore) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressScoreStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_egress_score_engine() -> EgressScoreEngine {
    EgressScoreEngine {
        steps: vec![
            Rc::new(MinimumEgressScoreAdjuster { minimum: 20.8 }) as Rc<dyn EgressScoreStep>,
            Rc::new(EgressScoreFloorValueOffsetAdjuster { weight: 0.49 }),
            Rc::new(RoundingEgressScoreAdjuster { step: 0.1 }),
            Rc::new(TieredEgressScoreAdjuster {
                bounds: vec![3443.0, 4985.0, 8933.0],
                rates: vec![1.08, 0.89, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 19527.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_score_engine();
        let item = new_egress_score("code-504", "label-313");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
