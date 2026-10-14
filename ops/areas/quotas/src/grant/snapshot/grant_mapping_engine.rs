use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the grant mapping calculation.
pub trait GrantMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMappingStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl GrantMappingAdjuster for GrantMappingStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &GrantMapping, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantMappingAdjuster for FlaggedGrantMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantMappingAdjuster {
    pub step: f64,
}

impl GrantMappingAdjuster for RoundingGrantMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantMappingAdjuster {
    pub minimum: f64,
}

impl GrantMappingAdjuster for MinimumGrantMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the grant mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantMappingEngine {
    pub steps: Vec<Rc<dyn GrantMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantMappingEngine {
    pub fn price(&mut self, item: &GrantMapping) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantMappingAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_mapping_engine(flags: &flags::Registry) -> GrantMappingEngine {
    GrantMappingEngine {
        steps: vec![
            Rc::new(GrantMappingStartsAtOffsetAdjuster { weight: 0.35 }) as Rc<dyn GrantMappingAdjuster>,
            Rc::new(FlaggedGrantMappingAdjuster {
                flags: flags.clone(),
                flag: "grant-mapping-regional".to_string(),
                percent: -4.3,
            }),
            Rc::new(RoundingGrantMappingAdjuster { step: 0.05 }),
            Rc::new(MinimumGrantMappingAdjuster { minimum: 44.4 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 65836.7,
    }
}
