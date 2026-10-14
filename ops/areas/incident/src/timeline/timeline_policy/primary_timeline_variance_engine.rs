use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary timeline variance calculation.
pub trait PrimaryTimelineVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTimelineVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryTimelineVarianceAdjuster {
    pub step: f64,
}

impl PrimaryTimelineVarianceStep for RoundingPrimaryTimelineVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryTimelineVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPrimaryTimelineVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryTimelineVarianceStep for UnitPrimaryTimelineVarianceAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PrimaryTimelineVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryTimelineVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryTimelineVarianceStep for FlaggedPrimaryTimelineVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryTimelineVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTimelineVarianceExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryTimelineVarianceStep for PrimaryTimelineVarianceExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryTimelineVariance, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryTimelineVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryTimelineVarianceStep for TieredPrimaryTimelineVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryTimelineVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryTimelineVarianceAdjuster {
    pub minimum: f64,
}

impl PrimaryTimelineVarianceStep for MinimumPrimaryTimelineVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryTimelineVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary timeline variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTimelineVarianceEngine {
    pub steps: Vec<Rc<dyn PrimaryTimelineVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTimelineVarianceEngine {
    pub fn run(&mut self, item: &PrimaryTimelineVariance) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryTimelineVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTimelineVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_timeline_variance_engine(flags: &flags::Registry) -> PrimaryTimelineVarianceEngine {
    PrimaryTimelineVarianceEngine {
        steps: vec![
            Rc::new(RoundingPrimaryTimelineVarianceAdjuster { step: 0.05 }) as Rc<dyn PrimaryTimelineVarianceStep>,
            Rc::new(UnitPrimaryTimelineVarianceAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.18)]),
            }),
            Rc::new(FlaggedPrimaryTimelineVarianceAdjuster {
                flags: flags.clone(),
                flag: "primary-timeline-variance-manual".to_string(),
                percent: -2.4,
            }),
            Rc::new(PrimaryTimelineVarianceExpiresAtOffsetAdjuster { weight: 0.61 }),
            Rc::new(TieredPrimaryTimelineVarianceAdjuster {
                bounds: vec![1314.0, 4087.0, 6497.0],
                rates: vec![0.84, 1.05, 1.11],
            }),
            Rc::new(MinimumPrimaryTimelineVarianceAdjuster { minimum: 14.1 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 88199.3,
    }
}
