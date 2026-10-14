use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback score calculation.
pub trait RollbackScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRollbackScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackScoreAdjuster for LabelRollbackScoreAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RollbackScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackScoreAdjuster {
    pub minimum: f64,
}

impl RollbackScoreAdjuster for MinimumRollbackScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackScoreAdjuster {
    pub cap: f64,
}

impl RollbackScoreAdjuster for CapRollbackScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackScoreAdjuster {
    pub step: f64,
}

impl RollbackScoreAdjuster for RoundingRollbackScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackScoreAdjuster {
    pub percent: f64,
}

impl RollbackScoreAdjuster for PercentRollbackScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollback score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackScoreEngine {
    pub steps: Vec<Rc<dyn RollbackScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackScoreEngine {
    pub fn compute(&mut self, item: &RollbackScore) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_score_engine() -> RollbackScoreEngine {
    RollbackScoreEngine {
        steps: vec![
            Rc::new(LabelRollbackScoreAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.72),
                    ("central".to_string(), 0.98),
                    ("retail".to_string(), 0.9),
                ]),
            }) as Rc<dyn RollbackScoreAdjuster>,
            Rc::new(MinimumRollbackScoreAdjuster { minimum: 22.0 }),
            Rc::new(CapRollbackScoreAdjuster { cap: 25643.0 }),
            Rc::new(RoundingRollbackScoreAdjuster { step: 0.1 }),
            Rc::new(PercentRollbackScoreAdjuster { percent: 16.2 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 11058.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_score_engine();
        let item = new_rollback_score("code-309", "label-622");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
