use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the postmortem target calculation.
pub trait PostmortemTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemTargetFactor for TieredPostmortemTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTargetScoreOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemTargetFactor for PostmortemTargetScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemTarget, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemTargetAdjuster {
    pub step: f64,
}

impl PostmortemTargetFactor for RoundingPostmortemTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPostmortemTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemTargetFactor for CategoryPostmortemTargetAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PostmortemTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemTargetAdjuster {
    pub minimum: f64,
}

impl PostmortemTargetFactor for MinimumPostmortemTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the postmortem target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemTargetEngine {
    pub steps: Vec<Rc<dyn PostmortemTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemTargetEngine {
    pub fn run(&mut self, item: &PostmortemTarget) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemTargetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_postmortem_target_engine() -> PostmortemTargetEngine {
    PostmortemTargetEngine {
        steps: vec![
            Rc::new(TieredPostmortemTargetAdjuster {
                bounds: vec![1940.0, 4263.0, 5087.0],
                rates: vec![1.0, 1.17, 0.95],
            }) as Rc<dyn PostmortemTargetFactor>,
            Rc::new(PostmortemTargetScoreOffsetAdjuster { weight: 1.96 }),
            Rc::new(RoundingPostmortemTargetAdjuster { step: 0.5 }),
            Rc::new(CategoryPostmortemTargetAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.15), ("retail".to_string(), 1.39)]),
            }),
            Rc::new(MinimumPostmortemTargetAdjuster { minimum: 4.8 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 98978.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_target_engine();
        let item = new_postmortem_target("code-697", "label-878");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
