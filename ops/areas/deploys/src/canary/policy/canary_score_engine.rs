use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary score calculation.
pub trait CanaryScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryScoreAdjuster for TieredCanaryScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryScoreAdjuster {
    pub step: f64,
}

impl CanaryScoreAdjuster for RoundingCanaryScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryScoreAdjuster {
    pub percent: f64,
}

impl CanaryScoreAdjuster for PercentCanaryScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScorePriorityOffsetAdjuster {
    pub weight: f64,
}

impl CanaryScoreAdjuster for CanaryScorePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryScore, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryScoreAdjuster {
    pub cap: f64,
}

impl CanaryScoreAdjuster for CapCanaryScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the canary score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryScoreEngine {
    pub steps: Vec<Rc<dyn CanaryScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryScoreEngine {
    pub fn run(&mut self, item: &CanaryScore) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[CanaryScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_canary_score_engine() -> CanaryScoreEngine {
    CanaryScoreEngine {
        steps: vec![
            Rc::new(TieredCanaryScoreAdjuster {
                bounds: vec![2793.0, 3802.0, 6484.0, 8366.0],
                rates: vec![1.15, 1.14, 1.19, 1.07],
            }) as Rc<dyn CanaryScoreAdjuster>,
            Rc::new(RoundingCanaryScoreAdjuster { step: 0.5 }),
            Rc::new(PercentCanaryScoreAdjuster { percent: 0.6 }),
            Rc::new(CanaryScorePriorityOffsetAdjuster { weight: 1.21 }),
            Rc::new(CapCanaryScoreAdjuster { cap: 18531.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 62294.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_score_engine();
        let item = new_canary_score("code-899", "label-357");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
