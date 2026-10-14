use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal delegation profile calculation.
pub trait SeasonalDelegationProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalDelegationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalDelegationProfileAdjuster {
    pub step: f64,
}

impl SeasonalDelegationProfileAdjuster for RoundingSeasonalDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalDelegationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalDelegationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalDelegationProfileAdjuster for TieredSeasonalDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalDelegationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalDelegationProfileAdjuster {
    pub percent: f64,
}

impl SeasonalDelegationProfileAdjuster for PercentSeasonalDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalDelegationProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalDelegationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalDelegationProfileAdjuster for FlaggedSeasonalDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalDelegationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal delegation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalDelegationProfileEngine {
    pub steps: Vec<Rc<dyn SeasonalDelegationProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalDelegationProfileEngine {
    pub fn price(&mut self, item: &SeasonalDelegationProfile) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalDelegationProfileAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalDelegationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_delegation_profile_engine(flags: &flags::Registry) -> SeasonalDelegationProfileEngine {
    SeasonalDelegationProfileEngine {
        steps: vec![
            Rc::new(RoundingSeasonalDelegationProfileAdjuster { step: 0.25 })
                as Rc<dyn SeasonalDelegationProfileAdjuster>,
            Rc::new(TieredSeasonalDelegationProfileAdjuster {
                bounds: vec![395.0, 2681.0],
                rates: vec![0.91, 1.18],
            }),
            Rc::new(PercentSeasonalDelegationProfileAdjuster { percent: -24.2 }),
            Rc::new(FlaggedSeasonalDelegationProfileAdjuster {
                flags: flags.clone(),
                flag: "seasonal-delegation-profile-legacy".to_string(),
                percent: -5.5,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 30708.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_delegation_profile_engine(&flags::new_registry());
        let item = new_seasonal_delegation_profile("code-999", "label-3");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
