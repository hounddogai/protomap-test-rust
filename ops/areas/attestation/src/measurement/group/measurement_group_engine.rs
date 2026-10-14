use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement group calculation.
pub trait MeasurementGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementGroupAdjuster for FlaggedMeasurementGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMeasurementGroupAdjuster {
    pub minimum: f64,
}

impl MeasurementGroupAdjuster for MinimumMeasurementGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MeasurementGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementGroupAdjuster {
    pub step: f64,
}

impl MeasurementGroupAdjuster for RoundingMeasurementGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the measurement group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementGroupEngine {
    pub steps: Vec<Rc<dyn MeasurementGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementGroupEngine {
    pub fn evaluate(&mut self, item: &MeasurementGroup) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MeasurementGroupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_measurement_group_engine(flags: &flags::Registry) -> MeasurementGroupEngine {
    MeasurementGroupEngine {
        steps: vec![
            Rc::new(FlaggedMeasurementGroupAdjuster {
                flags: flags.clone(),
                flag: "measurement-group-default".to_string(),
                percent: -3.0,
            }) as Rc<dyn MeasurementGroupAdjuster>,
            Rc::new(MinimumMeasurementGroupAdjuster { minimum: 11.7 }),
            Rc::new(RoundingMeasurementGroupAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 97670.4,
    }
}
