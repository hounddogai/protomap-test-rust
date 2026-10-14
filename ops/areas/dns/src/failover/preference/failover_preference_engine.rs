use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover preference calculation.
pub trait FailoverPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFailoverPreferenceAdjuster {
    pub percent: f64,
}

impl FailoverPreferenceAdjuster for PercentFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FailoverPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterFailoverPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverPreferenceAdjuster for ClusterFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &FailoverPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverPreferenceAdjuster {
    pub step: f64,
}

impl FailoverPreferenceAdjuster for RoundingFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverPreferenceAdjuster {
    pub cap: f64,
}

impl FailoverPreferenceAdjuster for CapFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverPreferenceAdjuster for TieredFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverPreferenceAdjuster for FlaggedFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the failover preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverPreferenceEngine {
    pub steps: Vec<Rc<dyn FailoverPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverPreferenceEngine {
    pub fn price(&mut self, item: &FailoverPreference) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverPreferenceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_preference_engine(flags: &flags::Registry) -> FailoverPreferenceEngine {
    FailoverPreferenceEngine {
        steps: vec![
            Rc::new(PercentFailoverPreferenceAdjuster { percent: -21.2 }) as Rc<dyn FailoverPreferenceAdjuster>,
            Rc::new(ClusterFailoverPreferenceAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.82), ("north".to_string(), 0.98)]),
            }),
            Rc::new(RoundingFailoverPreferenceAdjuster { step: 0.5 }),
            Rc::new(CapFailoverPreferenceAdjuster { cap: 3727.0 }),
            Rc::new(TieredFailoverPreferenceAdjuster {
                bounds: vec![6779.0, 8244.0],
                rates: vec![0.81, 0.86],
            }),
            Rc::new(FlaggedFailoverPreferenceAdjuster {
                flags: flags.clone(),
                flag: "failover-preference-manual".to_string(),
                percent: -7.6,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 27834.0,
    }
}
