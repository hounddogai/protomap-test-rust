use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention bundle calculation.
pub trait RetentionBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionBundleStep for FlaggedRetentionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionBundleAdjuster {
    pub step: f64,
}

impl RetentionBundleStep for RoundingRetentionBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRetentionBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionBundleStep for CategoryRetentionBundleAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RetentionBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionBundleAdjuster {
    pub minimum: f64,
}

impl RetentionBundleStep for MinimumRetentionBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the retention bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionBundleEngine {
    pub steps: Vec<Rc<dyn RetentionBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionBundleEngine {
    pub fn run(&mut self, item: &RetentionBundle) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_bundle_engine(flags: &flags::Registry) -> RetentionBundleEngine {
    RetentionBundleEngine {
        steps: vec![
            Rc::new(FlaggedRetentionBundleAdjuster {
                flags: flags.clone(),
                flag: "retention-bundle-scheduled".to_string(),
                percent: -0.3,
            }) as Rc<dyn RetentionBundleStep>,
            Rc::new(RoundingRetentionBundleAdjuster { step: 1.0 }),
            Rc::new(CategoryRetentionBundleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.0)]),
            }),
            Rc::new(MinimumRetentionBundleAdjuster { minimum: 34.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 3535.3,
    }
}
