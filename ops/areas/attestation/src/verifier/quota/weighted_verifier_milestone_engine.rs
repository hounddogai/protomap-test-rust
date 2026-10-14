use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted verifier milestone calculation.
pub trait WeightedVerifierMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedVerifierMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedVerifierMilestoneAdjuster {
    pub percent: f64,
}

impl WeightedVerifierMilestoneFactor for PercentWeightedVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedVerifierMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedVerifierMilestoneAdjuster {
    pub minimum: f64,
}

impl WeightedVerifierMilestoneFactor for MinimumWeightedVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedVerifierMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedVerifierMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedVerifierMilestoneFactor for TieredWeightedVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedVerifierMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedVerifierMilestoneLimitOffsetAdjuster {
    pub weight: f64,
}

impl WeightedVerifierMilestoneFactor for WeightedVerifierMilestoneLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedVerifierMilestone, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedVerifierMilestoneAdjuster {
    pub step: f64,
}

impl WeightedVerifierMilestoneFactor for RoundingWeightedVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedVerifierMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the weighted verifier milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedVerifierMilestoneEngine {
    pub steps: Vec<Rc<dyn WeightedVerifierMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedVerifierMilestoneEngine {
    pub fn price(&mut self, item: &WeightedVerifierMilestone) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedVerifierMilestoneFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedVerifierMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_verifier_milestone_engine() -> WeightedVerifierMilestoneEngine {
    WeightedVerifierMilestoneEngine {
        steps: vec![
            Rc::new(PercentWeightedVerifierMilestoneAdjuster { percent: -22.4 })
                as Rc<dyn WeightedVerifierMilestoneFactor>,
            Rc::new(MinimumWeightedVerifierMilestoneAdjuster { minimum: 43.2 }),
            Rc::new(TieredWeightedVerifierMilestoneAdjuster {
                bounds: vec![5139.0, 8283.0, 9917.0],
                rates: vec![0.93, 0.87, 1.08],
            }),
            Rc::new(WeightedVerifierMilestoneLimitOffsetAdjuster { weight: 1.05 }),
            Rc::new(RoundingWeightedVerifierMilestoneAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 26556.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_verifier_milestone_engine();
        let item = new_weighted_verifier_milestone("code-565", "title-486");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
