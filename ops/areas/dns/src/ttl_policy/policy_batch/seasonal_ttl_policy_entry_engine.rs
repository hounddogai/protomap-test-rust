use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the seasonal ttl policy entry calculation.
pub trait SeasonalTtlPolicyEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalTtlPolicyEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEntryRetriesOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalTtlPolicyEntryAdjuster for SeasonalTtlPolicyEntryRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyEntry, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalTtlPolicyEntryAdjuster {
    pub minimum: f64,
}

impl SeasonalTtlPolicyEntryAdjuster for MinimumSeasonalTtlPolicyEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelSeasonalTtlPolicyEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalTtlPolicyEntryAdjuster for ChannelSeasonalTtlPolicyEntryAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &SeasonalTtlPolicyEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalTtlPolicyEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalTtlPolicyEntryAdjuster for TieredSeasonalTtlPolicyEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal ttl policy entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalTtlPolicyEntryEngine {
    pub steps: Vec<Rc<dyn SeasonalTtlPolicyEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalTtlPolicyEntryEngine {
    pub fn compute(&mut self, item: &SeasonalTtlPolicyEntry) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalTtlPolicyEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalTtlPolicyEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_ttl_policy_entry_engine() -> SeasonalTtlPolicyEntryEngine {
    SeasonalTtlPolicyEntryEngine {
        steps: vec![
            Rc::new(SeasonalTtlPolicyEntryRetriesOffsetAdjuster { weight: 1.83 })
                as Rc<dyn SeasonalTtlPolicyEntryAdjuster>,
            Rc::new(MinimumSeasonalTtlPolicyEntryAdjuster { minimum: 41.6 }),
            Rc::new(ChannelSeasonalTtlPolicyEntryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.76)]),
            }),
            Rc::new(TieredSeasonalTtlPolicyEntryAdjuster {
                bounds: vec![5544.0, 5656.0],
                rates: vec![1.04, 1.09],
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 6803.0,
    }
}
