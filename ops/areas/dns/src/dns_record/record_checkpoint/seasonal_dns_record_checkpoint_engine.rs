use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal dns record checkpoint calculation.
pub trait SeasonalDnsRecordCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalDnsRecordCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalDnsRecordCheckpointAdjuster {
    pub cap: f64,
}

impl SeasonalDnsRecordCheckpointAdjuster for CapSeasonalDnsRecordCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalDnsRecordCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalDnsRecordCheckpointAdjuster {
    pub minimum: f64,
}

impl SeasonalDnsRecordCheckpointAdjuster for MinimumSeasonalDnsRecordCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalDnsRecordCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalDnsRecordCheckpointAdjuster {
    pub percent: f64,
}

impl SeasonalDnsRecordCheckpointAdjuster for PercentSeasonalDnsRecordCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalDnsRecordCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpointStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalDnsRecordCheckpointAdjuster for SeasonalDnsRecordCheckpointStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalDnsRecordCheckpoint, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalDnsRecordCheckpointAdjuster {
    pub step: f64,
}

impl SeasonalDnsRecordCheckpointAdjuster for RoundingSeasonalDnsRecordCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalDnsRecordCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalDnsRecordCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalDnsRecordCheckpointAdjuster for TieredSeasonalDnsRecordCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalDnsRecordCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal dns record checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalDnsRecordCheckpointEngine {
    pub steps: Vec<Rc<dyn SeasonalDnsRecordCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalDnsRecordCheckpointEngine {
    pub fn run(&mut self, item: &SeasonalDnsRecordCheckpoint) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalDnsRecordCheckpointAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalDnsRecordCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_dns_record_checkpoint_engine() -> SeasonalDnsRecordCheckpointEngine {
    SeasonalDnsRecordCheckpointEngine {
        steps: vec![
            Rc::new(CapSeasonalDnsRecordCheckpointAdjuster { cap: 29683.0 })
                as Rc<dyn SeasonalDnsRecordCheckpointAdjuster>,
            Rc::new(MinimumSeasonalDnsRecordCheckpointAdjuster { minimum: 2.1 }),
            Rc::new(PercentSeasonalDnsRecordCheckpointAdjuster { percent: 20.0 }),
            Rc::new(SeasonalDnsRecordCheckpointStartsAtOffsetAdjuster { weight: 1.37 }),
            Rc::new(RoundingSeasonalDnsRecordCheckpointAdjuster { step: 0.25 }),
            Rc::new(TieredSeasonalDnsRecordCheckpointAdjuster {
                bounds: vec![1514.0, 8158.0],
                rates: vec![1.18, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 11841.0,
    }
}
