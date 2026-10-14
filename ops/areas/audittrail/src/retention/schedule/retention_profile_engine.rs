use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention profile calculation.
pub trait RetentionProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionProfileAdjuster {
    pub step: f64,
}

impl RetentionProfileStep for RoundingRetentionProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionProfileStep for TieredRetentionProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionProfileAdjuster {
    pub minimum: f64,
}

impl RetentionProfileStep for MinimumRetentionProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionProfileStep for FlaggedRetentionProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the retention profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionProfileEngine {
    pub steps: Vec<Rc<dyn RetentionProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionProfileEngine {
    pub fn evaluate(&mut self, item: &RetentionProfile) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[RetentionProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_profile_engine(flags: &flags::Registry) -> RetentionProfileEngine {
    RetentionProfileEngine {
        steps: vec![
            Rc::new(RoundingRetentionProfileAdjuster { step: 0.5 }) as Rc<dyn RetentionProfileStep>,
            Rc::new(TieredRetentionProfileAdjuster {
                bounds: vec![1311.0, 3588.0, 5117.0, 6194.0],
                rates: vec![1.15, 0.87, 1.16, 0.85],
            }),
            Rc::new(MinimumRetentionProfileAdjuster { minimum: 35.0 }),
            Rc::new(FlaggedRetentionProfileAdjuster {
                flags: flags.clone(),
                flag: "retention-profile-tiered".to_string(),
                percent: 7.2,
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 54933.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_profile_engine(&flags::new_registry());
        let item = new_retention_profile("code-186", "title-107");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
