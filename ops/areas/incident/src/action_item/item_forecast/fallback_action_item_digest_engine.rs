use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback action item digest calculation.
pub trait FallbackActionItemDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackActionItemDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackActionItemDigestAdjuster {
    pub minimum: f64,
}

impl FallbackActionItemDigestStep for MinimumFallbackActionItemDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackActionItemDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemDigestRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl FallbackActionItemDigestStep for FallbackActionItemDigestRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackActionItemDigest, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackActionItemDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackActionItemDigestStep for TieredFallbackActionItemDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackActionItemDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackActionItemDigestAdjuster {
    pub step: f64,
}

impl FallbackActionItemDigestStep for RoundingFallbackActionItemDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackActionItemDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackActionItemDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackActionItemDigestStep for FlaggedFallbackActionItemDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackActionItemDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback action item digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackActionItemDigestEngine {
    pub steps: Vec<Rc<dyn FallbackActionItemDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackActionItemDigestEngine {
    pub fn run(&mut self, item: &FallbackActionItemDigest) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackActionItemDigestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackActionItemDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_action_item_digest_engine(flags: &flags::Registry) -> FallbackActionItemDigestEngine {
    FallbackActionItemDigestEngine {
        steps: vec![
            Rc::new(MinimumFallbackActionItemDigestAdjuster { minimum: 45.6 }) as Rc<dyn FallbackActionItemDigestStep>,
            Rc::new(FallbackActionItemDigestRotationDaysOffsetAdjuster { weight: 1.34 }),
            Rc::new(TieredFallbackActionItemDigestAdjuster {
                bounds: vec![789.0, 5233.0, 9040.0],
                rates: vec![1.06, 0.92, 1.16],
            }),
            Rc::new(RoundingFallbackActionItemDigestAdjuster { step: 5.0 }),
            Rc::new(FlaggedFallbackActionItemDigestAdjuster {
                flags: flags.clone(),
                flag: "fallback-action-item-digest-adaptive".to_string(),
                percent: -4.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 84769.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_action_item_digest_engine(&flags::new_registry());
        let item = new_fallback_action_item_digest("code-735", "label-239");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
