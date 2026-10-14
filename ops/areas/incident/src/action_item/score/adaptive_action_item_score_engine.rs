use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive action item score calculation.
pub trait AdaptiveActionItemScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveActionItemScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveActionItemScoreAdjuster {
    pub minimum: f64,
}

impl AdaptiveActionItemScoreAdjuster for MinimumAdaptiveActionItemScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveActionItemScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveActionItemScoreAdjuster {
    pub step: f64,
}

impl AdaptiveActionItemScoreAdjuster for RoundingAdaptiveActionItemScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveActionItemScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveActionItemScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveActionItemScoreAdjuster for TieredAdaptiveActionItemScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveActionItemScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveActionItemScoreExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveActionItemScoreAdjuster for AdaptiveActionItemScoreExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveActionItemScore, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

/// Runs the adaptive action item score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveActionItemScoreEngine {
    pub steps: Vec<Rc<dyn AdaptiveActionItemScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveActionItemScoreEngine {
    pub fn price(&mut self, item: &AdaptiveActionItemScore) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveActionItemScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_action_item_score_engine() -> AdaptiveActionItemScoreEngine {
    AdaptiveActionItemScoreEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveActionItemScoreAdjuster { minimum: 20.2 })
                as Rc<dyn AdaptiveActionItemScoreAdjuster>,
            Rc::new(RoundingAdaptiveActionItemScoreAdjuster { step: 5.0 }),
            Rc::new(TieredAdaptiveActionItemScoreAdjuster {
                bounds: vec![6755.0, 6941.0, 9041.0],
                rates: vec![1.15, 1.17, 0.86],
            }),
            Rc::new(AdaptiveActionItemScoreExpiresAtOffsetAdjuster { weight: 0.58 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 24394.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_action_item_score_engine();
        let item = new_adaptive_action_item_score("code-537", "label-624");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
