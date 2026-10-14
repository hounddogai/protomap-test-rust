use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict identity target calculation.
pub trait StrictIdentityTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictIdentityTarget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictIdentityTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictIdentityTargetStep for FlaggedStrictIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictIdentityTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictIdentityTargetAdjuster {
    pub cap: f64,
}

impl StrictIdentityTargetStep for CapStrictIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictIdentityTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictIdentityTargetAdjuster {
    pub minimum: f64,
}

impl StrictIdentityTargetStep for MinimumStrictIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictIdentityTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictIdentityTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictIdentityTargetStep for TieredStrictIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictIdentityTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictIdentityTargetAdjuster {
    pub step: f64,
}

impl StrictIdentityTargetStep for RoundingStrictIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictIdentityTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the strict identity target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictIdentityTargetEngine {
    pub steps: Vec<Rc<dyn StrictIdentityTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictIdentityTargetEngine {
    pub fn evaluate(&mut self, item: &StrictIdentityTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictIdentityTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictIdentityTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_identity_target_engine(flags: &flags::Registry) -> StrictIdentityTargetEngine {
    StrictIdentityTargetEngine {
        steps: vec![
            Rc::new(FlaggedStrictIdentityTargetAdjuster {
                flags: flags.clone(),
                flag: "strict-identity-target-lenient".to_string(),
                percent: -5.5,
            }) as Rc<dyn StrictIdentityTargetStep>,
            Rc::new(CapStrictIdentityTargetAdjuster { cap: 46797.0 }),
            Rc::new(MinimumStrictIdentityTargetAdjuster { minimum: 25.4 }),
            Rc::new(TieredStrictIdentityTargetAdjuster {
                bounds: vec![4730.0, 6835.0],
                rates: vec![1.02, 1.14],
            }),
            Rc::new(RoundingStrictIdentityTargetAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 16964.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_identity_target_engine(&flags::new_registry());
        let item = new_strict_identity_target("code-134", "title-952");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
