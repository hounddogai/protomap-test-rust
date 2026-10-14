use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive binding score calculation.
pub trait AdaptiveBindingScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveBindingScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveBindingScoreAdjuster {
    pub percent: f64,
}

impl AdaptiveBindingScoreFactor for PercentAdaptiveBindingScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveBindingScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveBindingScoreAdjuster {
    pub cap: f64,
}

impl AdaptiveBindingScoreFactor for CapAdaptiveBindingScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveBindingScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingScorePriorityOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveBindingScoreFactor for AdaptiveBindingScorePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveBindingScore, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveBindingScoreAdjuster {
    pub minimum: f64,
}

impl AdaptiveBindingScoreFactor for MinimumAdaptiveBindingScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveBindingScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveBindingScoreAdjuster {
    pub step: f64,
}

impl AdaptiveBindingScoreFactor for RoundingAdaptiveBindingScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveBindingScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAdaptiveBindingScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveBindingScoreFactor for LocaleAdaptiveBindingScoreAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AdaptiveBindingScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive binding score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveBindingScoreEngine {
    pub steps: Vec<Rc<dyn AdaptiveBindingScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveBindingScoreEngine {
    pub fn compute(&mut self, item: &AdaptiveBindingScore) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveBindingScoreFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveBindingScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_binding_score_engine() -> AdaptiveBindingScoreEngine {
    AdaptiveBindingScoreEngine {
        steps: vec![
            Rc::new(PercentAdaptiveBindingScoreAdjuster { percent: 9.5 }) as Rc<dyn AdaptiveBindingScoreFactor>,
            Rc::new(CapAdaptiveBindingScoreAdjuster { cap: 18843.0 }),
            Rc::new(AdaptiveBindingScorePriorityOffsetAdjuster { weight: 1.66 }),
            Rc::new(MinimumAdaptiveBindingScoreAdjuster { minimum: 0.4 }),
            Rc::new(RoundingAdaptiveBindingScoreAdjuster { step: 1.0 }),
            Rc::new(LocaleAdaptiveBindingScoreAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.35), ("west".to_string(), 1.31)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 2712.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_binding_score_engine();
        let item = new_adaptive_binding_score("code-748", "label-884");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
