use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default release threshold calculation.
pub trait DefaultReleaseThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultReleaseThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultReleaseThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultReleaseThresholdAdjuster for FlaggedDefaultReleaseThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultReleaseThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultReleaseThresholdAdjuster {
    pub minimum: f64,
}

impl DefaultReleaseThresholdAdjuster for MinimumDefaultReleaseThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultReleaseThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultReleaseThresholdAdjuster {
    pub cap: f64,
}

impl DefaultReleaseThresholdAdjuster for CapDefaultReleaseThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultReleaseThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDefaultReleaseThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultReleaseThresholdAdjuster for SegmentDefaultReleaseThresholdAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &DefaultReleaseThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultReleaseThresholdAdjuster {
    pub step: f64,
}

impl DefaultReleaseThresholdAdjuster for RoundingDefaultReleaseThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultReleaseThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default release threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultReleaseThresholdEngine {
    pub steps: Vec<Rc<dyn DefaultReleaseThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultReleaseThresholdEngine {
    pub fn run(&mut self, item: &DefaultReleaseThreshold) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultReleaseThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_release_threshold_engine(flags: &flags::Registry) -> DefaultReleaseThresholdEngine {
    DefaultReleaseThresholdEngine {
        steps: vec![
            Rc::new(FlaggedDefaultReleaseThresholdAdjuster {
                flags: flags.clone(),
                flag: "default-release-threshold-seasonal".to_string(),
                percent: 0.8,
            }) as Rc<dyn DefaultReleaseThresholdAdjuster>,
            Rc::new(MinimumDefaultReleaseThresholdAdjuster { minimum: 26.3 }),
            Rc::new(CapDefaultReleaseThresholdAdjuster { cap: 19560.0 }),
            Rc::new(SegmentDefaultReleaseThresholdAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.24), ("retail".to_string(), 1.29)]),
            }),
            Rc::new(RoundingDefaultReleaseThresholdAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 62669.7,
    }
}
