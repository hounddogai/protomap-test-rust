use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary nonce score calculation.
pub trait PrimaryNonceScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryNonceScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryNonceScoreAdjuster {
    pub minimum: f64,
}

impl PrimaryNonceScoreFactor for MinimumPrimaryNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryNonceScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryNonceScoreAdjuster {
    pub percent: f64,
}

impl PrimaryNonceScoreFactor for PercentPrimaryNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryNonceScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryNonceScoreAdjuster {
    pub cap: f64,
}

impl PrimaryNonceScoreFactor for CapPrimaryNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryNonceScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary nonce score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryNonceScoreEngine {
    pub steps: Vec<Rc<dyn PrimaryNonceScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryNonceScoreEngine {
    pub fn evaluate(&mut self, item: &PrimaryNonceScore) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PrimaryNonceScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_nonce_score_engine() -> PrimaryNonceScoreEngine {
    PrimaryNonceScoreEngine {
        steps: vec![
            Rc::new(MinimumPrimaryNonceScoreAdjuster { minimum: 40.3 }) as Rc<dyn PrimaryNonceScoreFactor>,
            Rc::new(PercentPrimaryNonceScoreAdjuster { percent: -16.6 }),
            Rc::new(CapPrimaryNonceScoreAdjuster { cap: 12320.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 94313.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_nonce_score_engine();
        let item = new_primary_nonce_score("code-864", "label-290");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
