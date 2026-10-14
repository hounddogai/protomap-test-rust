use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the seasonal region pin policy calculation.
pub trait SeasonalRegionPinPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalRegionPinPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalRegionPinPolicyAdjuster {
    pub cap: f64,
}

impl SeasonalRegionPinPolicyAdjuster for CapSeasonalRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalRegionPinPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalRegionPinPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalRegionPinPolicyAdjuster for TieredSeasonalRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalRegionPinPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalRegionPinPolicyAdjuster {
    pub minimum: f64,
}

impl SeasonalRegionPinPolicyAdjuster for MinimumSeasonalRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalRegionPinPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalRegionPinPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalRegionPinPolicyAdjuster for FlaggedSeasonalRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalRegionPinPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal region pin policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalRegionPinPolicyEngine {
    pub steps: Vec<Rc<dyn SeasonalRegionPinPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalRegionPinPolicyEngine {
    pub fn price(&mut self, item: &SeasonalRegionPinPolicy) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalRegionPinPolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalRegionPinPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_region_pin_policy_engine(flags: &flags::Registry) -> SeasonalRegionPinPolicyEngine {
    SeasonalRegionPinPolicyEngine {
        steps: vec![
            Rc::new(CapSeasonalRegionPinPolicyAdjuster { cap: 5114.0 }) as Rc<dyn SeasonalRegionPinPolicyAdjuster>,
            Rc::new(TieredSeasonalRegionPinPolicyAdjuster {
                bounds: vec![183.0, 1140.0, 3775.0],
                rates: vec![0.9, 0.83, 0.8],
            }),
            Rc::new(MinimumSeasonalRegionPinPolicyAdjuster { minimum: 25.1 }),
            Rc::new(FlaggedSeasonalRegionPinPolicyAdjuster {
                flags: flags.clone(),
                flag: "seasonal-region-pin-policy-fallback".to_string(),
                percent: 0.3,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 37302.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_region_pin_policy_engine(&flags::new_registry());
        let item = new_seasonal_region_pin_policy("code-345", "name-315");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
