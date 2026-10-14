use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient limit mapping calculation.
pub trait LenientLimitMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientLimitMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitMappingQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl LenientLimitMappingModifier for LenientLimitMappingQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &LenientLimitMapping, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientLimitMappingAdjuster {
    pub step: f64,
}

impl LenientLimitMappingModifier for RoundingLenientLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientLimitMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientLimitMappingAdjuster {
    pub minimum: f64,
}

impl LenientLimitMappingModifier for MinimumLenientLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientLimitMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient limit mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientLimitMappingEngine {
    pub steps: Vec<Rc<dyn LenientLimitMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientLimitMappingEngine {
    pub fn price(&mut self, item: &LenientLimitMapping) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[LenientLimitMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_limit_mapping_engine() -> LenientLimitMappingEngine {
    LenientLimitMappingEngine {
        steps: vec![
            Rc::new(LenientLimitMappingQuotaUnitsOffsetAdjuster { weight: 1.91 })
                as Rc<dyn LenientLimitMappingModifier>,
            Rc::new(RoundingLenientLimitMappingAdjuster { step: 5.0 }),
            Rc::new(MinimumLenientLimitMappingAdjuster { minimum: 22.8 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 93589.4,
    }
}
