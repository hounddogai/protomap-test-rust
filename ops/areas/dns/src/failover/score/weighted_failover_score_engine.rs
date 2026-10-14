use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted failover score calculation.
pub trait WeightedFailoverScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFailoverScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedFailoverScoreAdjuster {
    pub percent: f64,
}

impl WeightedFailoverScoreStep for PercentWeightedFailoverScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedFailoverScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverScoreRatioOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFailoverScoreStep for WeightedFailoverScoreRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFailoverScore, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFailoverScoreAdjuster {
    pub step: f64,
}

impl WeightedFailoverScoreStep for RoundingWeightedFailoverScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFailoverScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the weighted failover score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFailoverScoreEngine {
    pub steps: Vec<Rc<dyn WeightedFailoverScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFailoverScoreEngine {
    pub fn compute(&mut self, item: &WeightedFailoverScore) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFailoverScoreStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFailoverScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_failover_score_engine() -> WeightedFailoverScoreEngine {
    WeightedFailoverScoreEngine {
        steps: vec![
            Rc::new(PercentWeightedFailoverScoreAdjuster { percent: -18.5 }) as Rc<dyn WeightedFailoverScoreStep>,
            Rc::new(WeightedFailoverScoreRatioOffsetAdjuster { weight: 1.97 }),
            Rc::new(RoundingWeightedFailoverScoreAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 48765.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_failover_score_engine();
        let item = new_weighted_failover_score("code-670", "title-386");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
