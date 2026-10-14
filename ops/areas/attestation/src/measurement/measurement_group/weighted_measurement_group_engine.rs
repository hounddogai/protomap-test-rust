use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted measurement group calculation.
pub trait WeightedMeasurementGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedMeasurementGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedMeasurementGroupAdjuster {
    pub step: f64,
}

impl WeightedMeasurementGroupFactor for RoundingWeightedMeasurementGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedMeasurementGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameWeightedMeasurementGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedMeasurementGroupFactor for NameWeightedMeasurementGroupAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &WeightedMeasurementGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedMeasurementGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedMeasurementGroupFactor for FlaggedWeightedMeasurementGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedMeasurementGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedMeasurementGroupAdjuster {
    pub percent: f64,
}

impl WeightedMeasurementGroupFactor for PercentWeightedMeasurementGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedMeasurementGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted measurement group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedMeasurementGroupEngine {
    pub steps: Vec<Rc<dyn WeightedMeasurementGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedMeasurementGroupEngine {
    pub fn evaluate(&mut self, item: &WeightedMeasurementGroup) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedMeasurementGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_measurement_group_engine(flags: &flags::Registry) -> WeightedMeasurementGroupEngine {
    WeightedMeasurementGroupEngine {
        steps: vec![
            Rc::new(RoundingWeightedMeasurementGroupAdjuster { step: 0.5 }) as Rc<dyn WeightedMeasurementGroupFactor>,
            Rc::new(NameWeightedMeasurementGroupAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.17)]),
            }),
            Rc::new(FlaggedWeightedMeasurementGroupAdjuster {
                flags: flags.clone(),
                flag: "weighted-measurement-group-weighted".to_string(),
                percent: -5.0,
            }),
            Rc::new(PercentWeightedMeasurementGroupAdjuster { percent: 13.2 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 8649.7,
    }
}
