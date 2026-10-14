use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict release adjustment calculation.
pub trait StrictReleaseAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictReleaseAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameStrictReleaseAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictReleaseAdjustmentAdjuster for NameStrictReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &StrictReleaseAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictReleaseAdjustmentAdjuster {
    pub step: f64,
}

impl StrictReleaseAdjustmentAdjuster for RoundingStrictReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictReleaseAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictReleaseAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictReleaseAdjustmentAdjuster for FlaggedStrictReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictReleaseAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictReleaseAdjustmentAdjuster {
    pub percent: f64,
}

impl StrictReleaseAdjustmentAdjuster for PercentStrictReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictReleaseAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictReleaseAdjustmentAdjuster {
    pub cap: f64,
}

impl StrictReleaseAdjustmentAdjuster for CapStrictReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictReleaseAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictReleaseAdjustmentAdjuster {
    pub minimum: f64,
}

impl StrictReleaseAdjustmentAdjuster for MinimumStrictReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictReleaseAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict release adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictReleaseAdjustmentEngine {
    pub steps: Vec<Rc<dyn StrictReleaseAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictReleaseAdjustmentEngine {
    pub fn price(&mut self, item: &StrictReleaseAdjustment) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictReleaseAdjustmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictReleaseAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_release_adjustment_engine(flags: &flags::Registry) -> StrictReleaseAdjustmentEngine {
    StrictReleaseAdjustmentEngine {
        steps: vec![
            Rc::new(NameStrictReleaseAdjustmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.72)]),
            }) as Rc<dyn StrictReleaseAdjustmentAdjuster>,
            Rc::new(RoundingStrictReleaseAdjustmentAdjuster { step: 5.0 }),
            Rc::new(FlaggedStrictReleaseAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "strict-release-adjustment-lenient".to_string(),
                percent: 3.4,
            }),
            Rc::new(PercentStrictReleaseAdjustmentAdjuster { percent: 10.3 }),
            Rc::new(CapStrictReleaseAdjustmentAdjuster { cap: 1026.0 }),
            Rc::new(MinimumStrictReleaseAdjustmentAdjuster { minimum: 9.2 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 54610.4,
    }
}
