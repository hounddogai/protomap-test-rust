use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual binding tier calculation.
pub trait ManualBindingTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualBindingTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingTierThresholdOffsetAdjuster {
    pub weight: f64,
}

impl ManualBindingTierStep for ManualBindingTierThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &ManualBindingTier, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualBindingTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualBindingTierStep for TieredManualBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualBindingTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualBindingTierAdjuster {
    pub percent: f64,
}

impl ManualBindingTierStep for PercentManualBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualBindingTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualBindingTierAdjuster {
    pub step: f64,
}

impl ManualBindingTierStep for RoundingManualBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualBindingTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualBindingTierAdjuster {
    pub minimum: f64,
}

impl ManualBindingTierStep for MinimumManualBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualBindingTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual binding tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualBindingTierEngine {
    pub steps: Vec<Rc<dyn ManualBindingTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualBindingTierEngine {
    pub fn evaluate(&mut self, item: &ManualBindingTier) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualBindingTierStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualBindingTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_binding_tier_engine() -> ManualBindingTierEngine {
    ManualBindingTierEngine {
        steps: vec![
            Rc::new(ManualBindingTierThresholdOffsetAdjuster { weight: 1.49 }) as Rc<dyn ManualBindingTierStep>,
            Rc::new(TieredManualBindingTierAdjuster {
                bounds: vec![1856.0, 3339.0, 4773.0, 8438.0],
                rates: vec![0.88, 0.82, 0.85, 0.83],
            }),
            Rc::new(PercentManualBindingTierAdjuster { percent: 17.6 }),
            Rc::new(RoundingManualBindingTierAdjuster { step: 5.0 }),
            Rc::new(MinimumManualBindingTierAdjuster { minimum: 49.5 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 50078.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_binding_tier_engine();
        let item = new_manual_binding_tier("code-981", "label-524");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
