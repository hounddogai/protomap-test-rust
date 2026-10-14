use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the sink mapping calculation.
pub trait SinkMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkMappingMaximumOffsetAdjuster {
    pub weight: f64,
}

impl SinkMappingStep for SinkMappingMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &SinkMapping, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkMappingAdjuster {
    pub minimum: f64,
}

impl SinkMappingStep for MinimumSinkMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkMappingAdjuster {
    pub percent: f64,
}

impl SinkMappingStep for PercentSinkMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSinkMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkMappingStep for NameSinkMappingAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SinkMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the sink mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkMappingEngine {
    pub steps: Vec<Rc<dyn SinkMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkMappingEngine {
    pub fn compute(&mut self, item: &SinkMapping) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkMappingStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_sink_mapping_engine() -> SinkMappingEngine {
    SinkMappingEngine {
        steps: vec![
            Rc::new(SinkMappingMaximumOffsetAdjuster { weight: 1.13 }) as Rc<dyn SinkMappingStep>,
            Rc::new(MinimumSinkMappingAdjuster { minimum: 6.4 }),
            Rc::new(PercentSinkMappingAdjuster { percent: 8.8 }),
            Rc::new(NameSinkMappingAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.32),
                    ("online".to_string(), 0.99),
                    ("south".to_string(), 1.28),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 60685.9,
    }
}
