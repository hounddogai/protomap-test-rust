use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional zone override calculation.
pub trait RegionalZoneOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalZoneOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverrideFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RegionalZoneOverrideFactor for RegionalZoneOverrideFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalZoneOverride, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalZoneOverrideAdjuster {
    pub minimum: f64,
}

impl RegionalZoneOverrideFactor for MinimumRegionalZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalZoneOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalZoneOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalZoneOverrideFactor for FlaggedRegionalZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalZoneOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalZoneOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalZoneOverrideFactor for TieredRegionalZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalZoneOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRegionalZoneOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalZoneOverrideFactor for UnitRegionalZoneOverrideAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RegionalZoneOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalZoneOverrideAdjuster {
    pub cap: f64,
}

impl RegionalZoneOverrideFactor for CapRegionalZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalZoneOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional zone override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalZoneOverrideEngine {
    pub steps: Vec<Rc<dyn RegionalZoneOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalZoneOverrideEngine {
    pub fn run(&mut self, item: &RegionalZoneOverride) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalZoneOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_zone_override_engine(flags: &flags::Registry) -> RegionalZoneOverrideEngine {
    RegionalZoneOverrideEngine {
        steps: vec![
            Rc::new(RegionalZoneOverrideFloorValueOffsetAdjuster { weight: 0.48 })
                as Rc<dyn RegionalZoneOverrideFactor>,
            Rc::new(MinimumRegionalZoneOverrideAdjuster { minimum: 44.6 }),
            Rc::new(FlaggedRegionalZoneOverrideAdjuster {
                flags: flags.clone(),
                flag: "regional-zone-override-scheduled".to_string(),
                percent: -9.9,
            }),
            Rc::new(TieredRegionalZoneOverrideAdjuster {
                bounds: vec![5290.0, 5455.0, 7007.0, 8868.0],
                rates: vec![0.95, 0.84, 0.93, 0.88],
            }),
            Rc::new(UnitRegionalZoneOverrideAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.13), ("retail".to_string(), 1.36)]),
            }),
            Rc::new(CapRegionalZoneOverrideAdjuster { cap: 28294.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 63811.2,
    }
}
