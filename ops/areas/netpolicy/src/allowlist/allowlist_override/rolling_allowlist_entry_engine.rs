use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling allowlist entry calculation.
pub trait RollingAllowlistEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingAllowlistEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingAllowlistEntryAdjuster {
    pub minimum: f64,
}

impl RollingAllowlistEntryStep for MinimumRollingAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingAllowlistEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingAllowlistEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingAllowlistEntryStep for TieredRollingAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingAllowlistEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingAllowlistEntryAdjuster {
    pub step: f64,
}

impl RollingAllowlistEntryStep for RoundingRollingAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingAllowlistEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingAllowlistEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingAllowlistEntryStep for FlaggedRollingAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingAllowlistEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistEntryTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl RollingAllowlistEntryStep for RollingAllowlistEntryTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &RollingAllowlistEntry, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingAllowlistEntryAdjuster {
    pub cap: f64,
}

impl RollingAllowlistEntryStep for CapRollingAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingAllowlistEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling allowlist entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingAllowlistEntryEngine {
    pub steps: Vec<Rc<dyn RollingAllowlistEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingAllowlistEntryEngine {
    pub fn evaluate(&mut self, item: &RollingAllowlistEntry) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingAllowlistEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingAllowlistEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_allowlist_entry_engine(flags: &flags::Registry) -> RollingAllowlistEntryEngine {
    RollingAllowlistEntryEngine {
        steps: vec![
            Rc::new(MinimumRollingAllowlistEntryAdjuster { minimum: 27.5 }) as Rc<dyn RollingAllowlistEntryStep>,
            Rc::new(TieredRollingAllowlistEntryAdjuster {
                bounds: vec![1973.0, 9912.0],
                rates: vec![1.08, 1.15],
            }),
            Rc::new(RoundingRollingAllowlistEntryAdjuster { step: 5.0 }),
            Rc::new(FlaggedRollingAllowlistEntryAdjuster {
                flags: flags.clone(),
                flag: "rolling-allowlist-entry-capped".to_string(),
                percent: 1.5,
            }),
            Rc::new(RollingAllowlistEntryTtlSecondsOffsetAdjuster { weight: 1.0 }),
            Rc::new(CapRollingAllowlistEntryAdjuster { cap: 28316.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 18258.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_allowlist_entry_engine(&flags::new_registry());
        let item = new_rolling_allowlist_entry("code-356", "title-422");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
