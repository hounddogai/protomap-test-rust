use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global zone digest calculation.
pub trait GlobalZoneDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalZoneDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalZoneDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalZoneDigestStep for TieredGlobalZoneDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalZoneDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalZoneDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalZoneDigestStep for FlaggedGlobalZoneDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalZoneDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelGlobalZoneDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalZoneDigestStep for ChannelGlobalZoneDigestAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &GlobalZoneDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalZoneDigestFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl GlobalZoneDigestStep for GlobalZoneDigestFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalZoneDigest, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalZoneDigestAdjuster {
    pub percent: f64,
}

impl GlobalZoneDigestStep for PercentGlobalZoneDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalZoneDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global zone digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalZoneDigestEngine {
    pub steps: Vec<Rc<dyn GlobalZoneDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalZoneDigestEngine {
    pub fn price(&mut self, item: &GlobalZoneDigest) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalZoneDigestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalZoneDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_zone_digest_engine(flags: &flags::Registry) -> GlobalZoneDigestEngine {
    GlobalZoneDigestEngine {
        steps: vec![
            Rc::new(TieredGlobalZoneDigestAdjuster {
                bounds: vec![103.0, 3859.0, 7838.0],
                rates: vec![0.97, 0.84, 0.96],
            }) as Rc<dyn GlobalZoneDigestStep>,
            Rc::new(FlaggedGlobalZoneDigestAdjuster {
                flags: flags.clone(),
                flag: "global-zone-digest-batched".to_string(),
                percent: -7.8,
            }),
            Rc::new(ChannelGlobalZoneDigestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.92),
                    ("east".to_string(), 1.23),
                    ("retail".to_string(), 1.13),
                ]),
            }),
            Rc::new(GlobalZoneDigestFloorValueOffsetAdjuster { weight: 0.17 }),
            Rc::new(PercentGlobalZoneDigestAdjuster { percent: -24.1 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 12733.6,
    }
}
