use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query override calculation.
pub trait QueryOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryOverrideAdjuster {
    pub cap: f64,
}

impl QueryOverrideAdjuster for CapQueryOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryOverrideUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl QueryOverrideAdjuster for QueryOverrideUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &QueryOverride, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryOverrideAdjuster {
    pub step: f64,
}

impl QueryOverrideAdjuster for RoundingQueryOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryOverrideAdjuster {
    pub percent: f64,
}

impl QueryOverrideAdjuster for PercentQueryOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryOverrideAdjuster for FlaggedQueryOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryOverrideEngine {
    pub steps: Vec<Rc<dyn QueryOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryOverrideEngine {
    pub fn compute(&mut self, item: &QueryOverride) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[QueryOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_override_engine(flags: &flags::Registry) -> QueryOverrideEngine {
    QueryOverrideEngine {
        steps: vec![
            Rc::new(CapQueryOverrideAdjuster { cap: 2628.0 }) as Rc<dyn QueryOverrideAdjuster>,
            Rc::new(QueryOverrideUpdatedAtOffsetAdjuster { weight: 0.93 }),
            Rc::new(RoundingQueryOverrideAdjuster { step: 5.0 }),
            Rc::new(PercentQueryOverrideAdjuster { percent: 10.8 }),
            Rc::new(FlaggedQueryOverrideAdjuster {
                flags: flags.clone(),
                flag: "query-override-pending".to_string(),
                percent: -3.8,
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 77290.5,
    }
}
