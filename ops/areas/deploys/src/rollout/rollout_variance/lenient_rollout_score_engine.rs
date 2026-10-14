use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the lenient rollout score calculation.
pub trait LenientRolloutScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRolloutScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRolloutScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRolloutScoreAdjuster for TieredLenientRolloutScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRolloutScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientRolloutScoreAdjuster {
    pub minimum: f64,
}

impl LenientRolloutScoreAdjuster for MinimumLenientRolloutScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientRolloutScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutScoreAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl LenientRolloutScoreAdjuster for LenientRolloutScoreAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &LenientRolloutScore, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientRolloutScoreAdjuster {
    pub cap: f64,
}

impl LenientRolloutScoreAdjuster for CapLenientRolloutScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientRolloutScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient rollout score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRolloutScoreEngine {
    pub steps: Vec<Rc<dyn LenientRolloutScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRolloutScoreEngine {
    pub fn run(&mut self, item: &LenientRolloutScore) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientRolloutScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientRolloutScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_rollout_score_engine() -> LenientRolloutScoreEngine {
    LenientRolloutScoreEngine {
        steps: vec![
            Rc::new(TieredLenientRolloutScoreAdjuster {
                bounds: vec![1175.0, 1834.0, 8047.0],
                rates: vec![0.8, 1.11, 1.1],
            }) as Rc<dyn LenientRolloutScoreAdjuster>,
            Rc::new(MinimumLenientRolloutScoreAdjuster { minimum: 47.9 }),
            Rc::new(LenientRolloutScoreAttemptsOffsetAdjuster { weight: 1.64 }),
            Rc::new(CapLenientRolloutScoreAdjuster { cap: 24713.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 24469.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_rollout_score_engine();
        let item = new_lenient_rollout_score("code-576", "title-829");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
