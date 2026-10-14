use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict allowlist override calculation.
pub trait StrictAllowlistOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictAllowlistOverride, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictAllowlistOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictAllowlistOverrideAdjuster for FlaggedStrictAllowlistOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictAllowlistOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictAllowlistOverrideAdjuster {
    pub percent: f64,
}

impl StrictAllowlistOverrideAdjuster for PercentStrictAllowlistOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictAllowlistOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictAllowlistOverrideAdjuster {
    pub step: f64,
}

impl StrictAllowlistOverrideAdjuster for RoundingStrictAllowlistOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictAllowlistOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictAllowlistOverrideAdjuster {
    pub minimum: f64,
}

impl StrictAllowlistOverrideAdjuster for MinimumStrictAllowlistOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictAllowlistOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictAllowlistOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictAllowlistOverrideAdjuster for TieredStrictAllowlistOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictAllowlistOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict allowlist override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictAllowlistOverrideEngine {
    pub steps: Vec<Rc<dyn StrictAllowlistOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictAllowlistOverrideEngine {
    pub fn evaluate(&mut self, item: &StrictAllowlistOverride) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictAllowlistOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictAllowlistOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_allowlist_override_engine(flags: &flags::Registry) -> StrictAllowlistOverrideEngine {
    StrictAllowlistOverrideEngine {
        steps: vec![
            Rc::new(FlaggedStrictAllowlistOverrideAdjuster {
                flags: flags.clone(),
                flag: "strict-allowlist-override-weighted".to_string(),
                percent: -7.9,
            }) as Rc<dyn StrictAllowlistOverrideAdjuster>,
            Rc::new(PercentStrictAllowlistOverrideAdjuster { percent: 1.2 }),
            Rc::new(RoundingStrictAllowlistOverrideAdjuster { step: 0.5 }),
            Rc::new(MinimumStrictAllowlistOverrideAdjuster { minimum: 46.6 }),
            Rc::new(TieredStrictAllowlistOverrideAdjuster {
                bounds: vec![473.0, 1688.0, 4662.0],
                rates: vec![0.85, 0.93, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 81728.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_allowlist_override_engine(&flags::new_registry());
        let item = new_strict_allowlist_override("code-73", "name-801");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
