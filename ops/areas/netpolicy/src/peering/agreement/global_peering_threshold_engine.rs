use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global peering threshold calculation.
pub trait GlobalPeeringThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPeeringThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameGlobalPeeringThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalPeeringThresholdModifier for NameGlobalPeeringThresholdAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &GlobalPeeringThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPeeringThresholdAdjuster {
    pub cap: f64,
}

impl GlobalPeeringThresholdModifier for CapGlobalPeeringThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPeeringThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalPeeringThresholdAdjuster {
    pub percent: f64,
}

impl GlobalPeeringThresholdModifier for PercentGlobalPeeringThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalPeeringThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalPeeringThresholdAdjuster {
    pub step: f64,
}

impl GlobalPeeringThresholdModifier for RoundingGlobalPeeringThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalPeeringThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPeeringThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPeeringThresholdModifier for FlaggedGlobalPeeringThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPeeringThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global peering threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPeeringThresholdEngine {
    pub steps: Vec<Rc<dyn GlobalPeeringThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPeeringThresholdEngine {
    pub fn price(&mut self, item: &GlobalPeeringThreshold) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalPeeringThresholdModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPeeringThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_peering_threshold_engine(flags: &flags::Registry) -> GlobalPeeringThresholdEngine {
    GlobalPeeringThresholdEngine {
        steps: vec![
            Rc::new(NameGlobalPeeringThresholdAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.2)]),
            }) as Rc<dyn GlobalPeeringThresholdModifier>,
            Rc::new(CapGlobalPeeringThresholdAdjuster { cap: 42831.0 }),
            Rc::new(PercentGlobalPeeringThresholdAdjuster { percent: -21.2 }),
            Rc::new(RoundingGlobalPeeringThresholdAdjuster { step: 5.0 }),
            Rc::new(FlaggedGlobalPeeringThresholdAdjuster {
                flags: flags.clone(),
                flag: "global-peering-threshold-adaptive".to_string(),
                percent: 2.8,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 86174.3,
    }
}
