use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted identity record calculation.
pub trait WeightedIdentityRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedIdentityRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryWeightedIdentityRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedIdentityRecordAdjuster for CategoryWeightedIdentityRecordAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &WeightedIdentityRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedIdentityRecordAdjuster {
    pub step: f64,
}

impl WeightedIdentityRecordAdjuster for RoundingWeightedIdentityRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedIdentityRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedIdentityRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedIdentityRecordAdjuster for FlaggedWeightedIdentityRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedIdentityRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted identity record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedIdentityRecordEngine {
    pub steps: Vec<Rc<dyn WeightedIdentityRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedIdentityRecordEngine {
    pub fn run(&mut self, item: &WeightedIdentityRecord) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedIdentityRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedIdentityRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_identity_record_engine(flags: &flags::Registry) -> WeightedIdentityRecordEngine {
    WeightedIdentityRecordEngine {
        steps: vec![
            Rc::new(CategoryWeightedIdentityRecordAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.95),
                    ("online".to_string(), 0.94),
                    ("central".to_string(), 0.72),
                ]),
            }) as Rc<dyn WeightedIdentityRecordAdjuster>,
            Rc::new(RoundingWeightedIdentityRecordAdjuster { step: 0.05 }),
            Rc::new(FlaggedWeightedIdentityRecordAdjuster {
                flags: flags.clone(),
                flag: "weighted-identity-record-scheduled".to_string(),
                percent: -5.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 84796.1,
    }
}
