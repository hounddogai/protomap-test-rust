use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped usage profile calculation.
pub trait CappedUsageProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedUsageProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedUsageProfileAdjuster {
    pub minimum: f64,
}

impl CappedUsageProfileFactor for MinimumCappedUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedUsageProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedUsageProfileAdjuster {
    pub step: f64,
}

impl CappedUsageProfileFactor for RoundingCappedUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedUsageProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedUsageProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedUsageProfileFactor for FlaggedCappedUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedUsageProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedUsageProfileAdjuster {
    pub cap: f64,
}

impl CappedUsageProfileFactor for CapCappedUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedUsageProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedUsageProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedUsageProfileFactor for TieredCappedUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedUsageProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelCappedUsageProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedUsageProfileFactor for ChannelCappedUsageProfileAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &CappedUsageProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the capped usage profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedUsageProfileEngine {
    pub steps: Vec<Rc<dyn CappedUsageProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedUsageProfileEngine {
    pub fn price(&mut self, item: &CappedUsageProfile) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[CappedUsageProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_usage_profile_engine(flags: &flags::Registry) -> CappedUsageProfileEngine {
    CappedUsageProfileEngine {
        steps: vec![
            Rc::new(MinimumCappedUsageProfileAdjuster { minimum: 9.1 }) as Rc<dyn CappedUsageProfileFactor>,
            Rc::new(RoundingCappedUsageProfileAdjuster { step: 5.0 }),
            Rc::new(FlaggedCappedUsageProfileAdjuster {
                flags: flags.clone(),
                flag: "capped-usage-profile-strict".to_string(),
                percent: -4.9,
            }),
            Rc::new(CapCappedUsageProfileAdjuster { cap: 20843.0 }),
            Rc::new(TieredCappedUsageProfileAdjuster {
                bounds: vec![296.0, 657.0, 1994.0, 5681.0],
                rates: vec![1.05, 1.05, 0.96, 1.18],
            }),
            Rc::new(ChannelCappedUsageProfileAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.2)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 38762.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_usage_profile_engine(&flags::new_registry());
        let item = new_capped_usage_profile("code-507", "label-726");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
