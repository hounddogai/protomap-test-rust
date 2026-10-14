use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the zone quota calculation.
pub trait ZoneQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneQuotaAdjuster {
    pub minimum: f64,
}

impl ZoneQuotaStep for MinimumZoneQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelZoneQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneQuotaStep for ChannelZoneQuotaAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ZoneQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneQuotaStep for FlaggedZoneQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneQuotaCeilingOffsetAdjuster {
    pub weight: f64,
}

impl ZoneQuotaStep for ZoneQuotaCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneQuota, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneQuotaAdjuster {
    pub percent: f64,
}

impl ZoneQuotaStep for PercentZoneQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneQuotaAdjuster {
    pub step: f64,
}

impl ZoneQuotaStep for RoundingZoneQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the zone quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneQuotaEngine {
    pub steps: Vec<Rc<dyn ZoneQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneQuotaEngine {
    pub fn evaluate(&mut self, item: &ZoneQuota) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneQuotaStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_zone_quota_engine(flags: &flags::Registry) -> ZoneQuotaEngine {
    ZoneQuotaEngine {
        steps: vec![
            Rc::new(MinimumZoneQuotaAdjuster { minimum: 43.4 }) as Rc<dyn ZoneQuotaStep>,
            Rc::new(ChannelZoneQuotaAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.85), ("north".to_string(), 1.23)]),
            }),
            Rc::new(FlaggedZoneQuotaAdjuster {
                flags: flags.clone(),
                flag: "zone-quota-regional".to_string(),
                percent: -1.8,
            }),
            Rc::new(ZoneQuotaCeilingOffsetAdjuster { weight: 0.57 }),
            Rc::new(PercentZoneQuotaAdjuster { percent: -13.5 }),
            Rc::new(RoundingZoneQuotaAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 26615.3,
    }
}
