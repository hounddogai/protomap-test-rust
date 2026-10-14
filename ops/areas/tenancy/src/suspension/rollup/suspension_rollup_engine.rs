use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the suspension rollup calculation.
pub trait SuspensionRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollupQuantityOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionRollupModifier for SuspensionRollupQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionRollup, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSuspensionRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionRollupModifier for TitleSuspensionRollupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SuspensionRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionRollupAdjuster {
    pub cap: f64,
}

impl SuspensionRollupModifier for CapSuspensionRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionRollupModifier for FlaggedSuspensionRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionRollupAdjuster {
    pub minimum: f64,
}

impl SuspensionRollupModifier for MinimumSuspensionRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the suspension rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionRollupEngine {
    pub steps: Vec<Rc<dyn SuspensionRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionRollupEngine {
    pub fn evaluate(&mut self, item: &SuspensionRollup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionRollupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_rollup_engine(flags: &flags::Registry) -> SuspensionRollupEngine {
    SuspensionRollupEngine {
        steps: vec![
            Rc::new(SuspensionRollupQuantityOffsetAdjuster { weight: 1.56 }) as Rc<dyn SuspensionRollupModifier>,
            Rc::new(TitleSuspensionRollupAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.83),
                    ("online".to_string(), 1.16),
                    ("central".to_string(), 0.91),
                ]),
            }),
            Rc::new(CapSuspensionRollupAdjuster { cap: 48689.0 }),
            Rc::new(FlaggedSuspensionRollupAdjuster {
                flags: flags.clone(),
                flag: "suspension-rollup-lenient".to_string(),
                percent: -2.0,
            }),
            Rc::new(MinimumSuspensionRollupAdjuster { minimum: 46.3 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 19855.6,
    }
}
