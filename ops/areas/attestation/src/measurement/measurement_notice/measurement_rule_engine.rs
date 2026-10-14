use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement rule calculation.
pub trait MeasurementRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementRuleFactor for FlaggedMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRuleMaximumOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementRuleFactor for MeasurementRuleMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementRule, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementRuleFactor for TieredMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the measurement rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementRuleEngine {
    pub steps: Vec<Rc<dyn MeasurementRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementRuleEngine {
    pub fn run(&mut self, item: &MeasurementRule) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_measurement_rule_engine(flags: &flags::Registry) -> MeasurementRuleEngine {
    MeasurementRuleEngine {
        steps: vec![
            Rc::new(FlaggedMeasurementRuleAdjuster {
                flags: flags.clone(),
                flag: "measurement-rule-pending".to_string(),
                percent: 1.0,
            }) as Rc<dyn MeasurementRuleFactor>,
            Rc::new(MeasurementRuleMaximumOffsetAdjuster { weight: 0.89 }),
            Rc::new(TieredMeasurementRuleAdjuster {
                bounds: vec![6074.0, 6129.0, 7426.0, 9598.0],
                rates: vec![0.92, 1.17, 0.93, 1.14],
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 15304.1,
    }
}
