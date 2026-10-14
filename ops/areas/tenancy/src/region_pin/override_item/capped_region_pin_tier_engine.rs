use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped region pin tier calculation.
pub trait CappedRegionPinTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRegionPinTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinTierVersionOffsetAdjuster {
    pub weight: f64,
}

impl CappedRegionPinTierModifier for CappedRegionPinTierVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &CappedRegionPinTier, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedRegionPinTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedRegionPinTierModifier for NameCappedRegionPinTierAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedRegionPinTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedRegionPinTierAdjuster {
    pub cap: f64,
}

impl CappedRegionPinTierModifier for CapCappedRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedRegionPinTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedRegionPinTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedRegionPinTierModifier for FlaggedCappedRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedRegionPinTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedRegionPinTierAdjuster {
    pub percent: f64,
}

impl CappedRegionPinTierModifier for PercentCappedRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedRegionPinTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped region pin tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRegionPinTierEngine {
    pub steps: Vec<Rc<dyn CappedRegionPinTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRegionPinTierEngine {
    pub fn price(&mut self, item: &CappedRegionPinTier) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedRegionPinTierModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedRegionPinTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_region_pin_tier_engine(flags: &flags::Registry) -> CappedRegionPinTierEngine {
    CappedRegionPinTierEngine {
        steps: vec![
            Rc::new(CappedRegionPinTierVersionOffsetAdjuster { weight: 0.79 }) as Rc<dyn CappedRegionPinTierModifier>,
            Rc::new(NameCappedRegionPinTierAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.92)]),
            }),
            Rc::new(CapCappedRegionPinTierAdjuster { cap: 37025.0 }),
            Rc::new(FlaggedCappedRegionPinTierAdjuster {
                flags: flags.clone(),
                flag: "capped-region-pin-tier-pending".to_string(),
                percent: 7.0,
            }),
            Rc::new(PercentCappedRegionPinTierAdjuster { percent: -21.8 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 75423.7,
    }
}
