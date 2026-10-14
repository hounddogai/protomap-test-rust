use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit tier calculation.
pub trait LimitTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitTierAdjuster {
    pub step: f64,
}

impl LimitTierAdjuster for RoundingLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitTierAdjuster {
    pub percent: f64,
}

impl LimitTierAdjuster for PercentLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitTierAdjuster {
    pub minimum: f64,
}

impl LimitTierAdjuster for MinimumLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the limit tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitTierEngine {
    pub steps: Vec<Rc<dyn LimitTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitTierEngine {
    pub fn compute(&mut self, item: &LimitTier) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[LimitTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_limit_tier_engine() -> LimitTierEngine {
    LimitTierEngine {
        steps: vec![
            Rc::new(RoundingLimitTierAdjuster { step: 5.0 }) as Rc<dyn LimitTierAdjuster>,
            Rc::new(PercentLimitTierAdjuster { percent: -9.9 }),
            Rc::new(MinimumLimitTierAdjuster { minimum: 14.5 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 47858.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_tier_engine();
        let item = new_limit_tier("code-155", "name-815");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
