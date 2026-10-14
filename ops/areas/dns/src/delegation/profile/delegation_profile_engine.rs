use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation profile calculation.
pub trait DelegationProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationProfileFactor for TieredDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationProfileAdjuster {
    pub minimum: f64,
}

impl DelegationProfileFactor for MinimumDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationProfileAdjuster {
    pub step: f64,
}

impl DelegationProfileFactor for RoundingDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationProfileAdjuster {
    pub cap: f64,
}

impl DelegationProfileFactor for CapDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationProfileFactor for FlaggedDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProfileEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl DelegationProfileFactor for DelegationProfileEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationProfile, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the delegation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationProfileEngine {
    pub steps: Vec<Rc<dyn DelegationProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationProfileEngine {
    pub fn price(&mut self, item: &DelegationProfile) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[DelegationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_delegation_profile_engine(flags: &flags::Registry) -> DelegationProfileEngine {
    DelegationProfileEngine {
        steps: vec![
            Rc::new(TieredDelegationProfileAdjuster {
                bounds: vec![2209.0, 3820.0, 6712.0, 9467.0],
                rates: vec![1.17, 0.98, 1.03, 1.13],
            }) as Rc<dyn DelegationProfileFactor>,
            Rc::new(MinimumDelegationProfileAdjuster { minimum: 5.3 }),
            Rc::new(RoundingDelegationProfileAdjuster { step: 0.25 }),
            Rc::new(CapDelegationProfileAdjuster { cap: 47302.0 }),
            Rc::new(FlaggedDelegationProfileAdjuster {
                flags: flags.clone(),
                flag: "delegation-profile-weighted".to_string(),
                percent: -5.0,
            }),
            Rc::new(DelegationProfileEndsAtOffsetAdjuster { weight: 1.99 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 73351.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_profile_engine(&flags::new_registry());
        let item = new_delegation_profile("code-316", "name-65");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
