use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention mapping calculation.
pub trait RetentionMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionMapping, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionMappingModifier for FlaggedRetentionMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionMappingAdjuster {
    pub minimum: f64,
}

impl RetentionMappingModifier for MinimumRetentionMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionMappingAdjuster {
    pub cap: f64,
}

impl RetentionMappingModifier for CapRetentionMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRetentionMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionMappingModifier for RegionRetentionMappingAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RetentionMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMappingSequenceOffsetAdjuster {
    pub weight: f64,
}

impl RetentionMappingModifier for RetentionMappingSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionMapping, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the retention mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionMappingEngine {
    pub steps: Vec<Rc<dyn RetentionMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionMappingEngine {
    pub fn compute(&mut self, item: &RetentionMapping) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RetentionMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_retention_mapping_engine(flags: &flags::Registry) -> RetentionMappingEngine {
    RetentionMappingEngine {
        steps: vec![
            Rc::new(FlaggedRetentionMappingAdjuster {
                flags: flags.clone(),
                flag: "retention-mapping-strict".to_string(),
                percent: -7.2,
            }) as Rc<dyn RetentionMappingModifier>,
            Rc::new(MinimumRetentionMappingAdjuster { minimum: 12.2 }),
            Rc::new(CapRetentionMappingAdjuster { cap: 2589.0 }),
            Rc::new(RegionRetentionMappingAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.8),
                    ("retail".to_string(), 1.32),
                    ("east".to_string(), 0.97),
                ]),
            }),
            Rc::new(RetentionMappingSequenceOffsetAdjuster { weight: 1.58 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 83578.4,
    }
}
