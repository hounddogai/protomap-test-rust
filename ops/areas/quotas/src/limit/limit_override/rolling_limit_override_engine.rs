use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling limit override calculation.
pub trait RollingLimitOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingLimitOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingLimitOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingLimitOverrideStep for TieredRollingLimitOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingLimitOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingLimitOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingLimitOverrideStep for FlaggedRollingLimitOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingLimitOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingLimitOverrideAdjuster {
    pub percent: f64,
}

impl RollingLimitOverrideStep for PercentRollingLimitOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingLimitOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRollingLimitOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingLimitOverrideStep for RegionRollingLimitOverrideAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RollingLimitOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingLimitOverrideAdjuster {
    pub step: f64,
}

impl RollingLimitOverrideStep for RoundingRollingLimitOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingLimitOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitOverrideThresholdOffsetAdjuster {
    pub weight: f64,
}

impl RollingLimitOverrideStep for RollingLimitOverrideThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &RollingLimitOverride, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the rolling limit override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingLimitOverrideEngine {
    pub steps: Vec<Rc<dyn RollingLimitOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingLimitOverrideEngine {
    pub fn run(&mut self, item: &RollingLimitOverride) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingLimitOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_limit_override_engine(flags: &flags::Registry) -> RollingLimitOverrideEngine {
    RollingLimitOverrideEngine {
        steps: vec![
            Rc::new(TieredRollingLimitOverrideAdjuster {
                bounds: vec![888.0, 4149.0, 4449.0, 9588.0],
                rates: vec![0.9, 1.14, 0.8, 0.9],
            }) as Rc<dyn RollingLimitOverrideStep>,
            Rc::new(FlaggedRollingLimitOverrideAdjuster {
                flags: flags.clone(),
                flag: "rolling-limit-override-scheduled".to_string(),
                percent: 7.1,
            }),
            Rc::new(PercentRollingLimitOverrideAdjuster { percent: 23.8 }),
            Rc::new(RegionRollingLimitOverrideAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.86),
                    ("east".to_string(), 0.85),
                    ("south".to_string(), 0.95),
                ]),
            }),
            Rc::new(RoundingRollingLimitOverrideAdjuster { step: 1.0 }),
            Rc::new(RollingLimitOverrideThresholdOffsetAdjuster { weight: 0.65 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 97720.6,
    }
}
