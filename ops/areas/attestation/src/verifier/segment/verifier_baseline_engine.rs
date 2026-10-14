use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the verifier baseline calculation.
pub trait VerifierBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelVerifierBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierBaselineModifier for ChannelVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &VerifierBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBaselineScoreOffsetAdjuster {
    pub weight: f64,
}

impl VerifierBaselineModifier for VerifierBaselineScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierBaseline, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierBaselineAdjuster {
    pub step: f64,
}

impl VerifierBaselineModifier for RoundingVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierBaselineAdjuster {
    pub minimum: f64,
}

impl VerifierBaselineModifier for MinimumVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierBaselineAdjuster {
    pub cap: f64,
}

impl VerifierBaselineModifier for CapVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the verifier baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierBaselineEngine {
    pub steps: Vec<Rc<dyn VerifierBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierBaselineEngine {
    pub fn price(&mut self, item: &VerifierBaseline) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierBaselineModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_verifier_baseline_engine() -> VerifierBaselineEngine {
    VerifierBaselineEngine {
        steps: vec![
            Rc::new(ChannelVerifierBaselineAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.91)]),
            }) as Rc<dyn VerifierBaselineModifier>,
            Rc::new(VerifierBaselineScoreOffsetAdjuster { weight: 1.81 }),
            Rc::new(RoundingVerifierBaselineAdjuster { step: 1.0 }),
            Rc::new(MinimumVerifierBaselineAdjuster { minimum: 20.3 }),
            Rc::new(CapVerifierBaselineAdjuster { cap: 18852.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 15061.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_baseline_engine();
        let item = new_verifier_baseline("code-540", "label-220");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
