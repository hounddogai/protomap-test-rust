use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze override calculation.
pub trait FreezeOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeOverrideAdjuster {
    pub cap: f64,
}

impl FreezeOverrideStep for CapFreezeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeOverrideAdjuster {
    pub minimum: f64,
}

impl FreezeOverrideStep for MinimumFreezeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFreezeOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeOverrideStep for ChannelFreezeOverrideAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &FreezeOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeOverrideAdjuster {
    pub step: f64,
}

impl FreezeOverrideStep for RoundingFreezeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeOverrideStep for TieredFreezeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeOverrideStep for FlaggedFreezeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the freeze override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeOverrideEngine {
    pub steps: Vec<Rc<dyn FreezeOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeOverrideEngine {
    pub fn evaluate(&mut self, item: &FreezeOverride) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeOverrideStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_freeze_override_engine(flags: &flags::Registry) -> FreezeOverrideEngine {
    FreezeOverrideEngine {
        steps: vec![
            Rc::new(CapFreezeOverrideAdjuster { cap: 32909.0 }) as Rc<dyn FreezeOverrideStep>,
            Rc::new(MinimumFreezeOverrideAdjuster { minimum: 2.9 }),
            Rc::new(ChannelFreezeOverrideAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.34)]),
            }),
            Rc::new(RoundingFreezeOverrideAdjuster { step: 0.25 }),
            Rc::new(TieredFreezeOverrideAdjuster {
                bounds: vec![5377.0, 6192.0],
                rates: vec![0.99, 0.9],
            }),
            Rc::new(FlaggedFreezeOverrideAdjuster {
                flags: flags.clone(),
                flag: "freeze-override-legacy".to_string(),
                percent: 5.5,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 41533.3,
    }
}
