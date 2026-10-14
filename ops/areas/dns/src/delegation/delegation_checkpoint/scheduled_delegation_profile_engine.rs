use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled delegation profile calculation.
pub trait ScheduledDelegationProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDelegationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledDelegationProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledDelegationProfileStep for TitleScheduledDelegationProfileAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledDelegationProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledDelegationProfileAdjuster {
    pub cap: f64,
}

impl ScheduledDelegationProfileStep for CapScheduledDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledDelegationProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledDelegationProfileAdjuster {
    pub minimum: f64,
}

impl ScheduledDelegationProfileStep for MinimumScheduledDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledDelegationProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledDelegationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledDelegationProfileStep for FlaggedScheduledDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledDelegationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledDelegationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledDelegationProfileStep for TieredScheduledDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledDelegationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledDelegationProfileAdjuster {
    pub step: f64,
}

impl ScheduledDelegationProfileStep for RoundingScheduledDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledDelegationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled delegation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDelegationProfileEngine {
    pub steps: Vec<Rc<dyn ScheduledDelegationProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDelegationProfileEngine {
    pub fn price(&mut self, item: &ScheduledDelegationProfile) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledDelegationProfileStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDelegationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_delegation_profile_engine(flags: &flags::Registry) -> ScheduledDelegationProfileEngine {
    ScheduledDelegationProfileEngine {
        steps: vec![
            Rc::new(TitleScheduledDelegationProfileAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.29), ("retail".to_string(), 0.89)]),
            }) as Rc<dyn ScheduledDelegationProfileStep>,
            Rc::new(CapScheduledDelegationProfileAdjuster { cap: 36700.0 }),
            Rc::new(MinimumScheduledDelegationProfileAdjuster { minimum: 12.4 }),
            Rc::new(FlaggedScheduledDelegationProfileAdjuster {
                flags: flags.clone(),
                flag: "scheduled-delegation-profile-scheduled".to_string(),
                percent: -2.4,
            }),
            Rc::new(TieredScheduledDelegationProfileAdjuster {
                bounds: vec![2233.0, 3983.0, 5950.0, 7526.0],
                rates: vec![0.97, 0.96, 1.1, 1.07],
            }),
            Rc::new(RoundingScheduledDelegationProfileAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 18262.6,
    }
}
