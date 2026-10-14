use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback zone threshold calculation.
pub trait FallbackZoneThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackZoneThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackZoneThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackZoneThresholdAdjuster for FlaggedFallbackZoneThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackZoneThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackZoneThresholdAdjuster {
    pub step: f64,
}

impl FallbackZoneThresholdAdjuster for RoundingFallbackZoneThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackZoneThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackZoneThresholdAdjuster {
    pub cap: f64,
}

impl FallbackZoneThresholdAdjuster for CapFallbackZoneThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackZoneThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentFallbackZoneThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackZoneThresholdAdjuster for SegmentFallbackZoneThresholdAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &FallbackZoneThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackZoneThresholdAdjuster {
    pub minimum: f64,
}

impl FallbackZoneThresholdAdjuster for MinimumFallbackZoneThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackZoneThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback zone threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackZoneThresholdEngine {
    pub steps: Vec<Rc<dyn FallbackZoneThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackZoneThresholdEngine {
    pub fn evaluate(&mut self, item: &FallbackZoneThreshold) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackZoneThresholdAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackZoneThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_zone_threshold_engine(flags: &flags::Registry) -> FallbackZoneThresholdEngine {
    FallbackZoneThresholdEngine {
        steps: vec![
            Rc::new(FlaggedFallbackZoneThresholdAdjuster {
                flags: flags.clone(),
                flag: "fallback-zone-threshold-adaptive".to_string(),
                percent: -0.3,
            }) as Rc<dyn FallbackZoneThresholdAdjuster>,
            Rc::new(RoundingFallbackZoneThresholdAdjuster { step: 1.0 }),
            Rc::new(CapFallbackZoneThresholdAdjuster { cap: 19929.0 }),
            Rc::new(SegmentFallbackZoneThresholdAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.18), ("retail".to_string(), 0.86)]),
            }),
            Rc::new(MinimumFallbackZoneThresholdAdjuster { minimum: 37.8 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 54440.6,
    }
}
