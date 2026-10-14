use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction schedule calculation.
pub trait RedactionScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRedactionScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionScheduleModifier for UnitRedactionScheduleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RedactionSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScheduleScoreOffsetAdjuster {
    pub weight: f64,
}

impl RedactionScheduleModifier for RedactionScheduleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionSchedule, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionScheduleModifier for TieredRedactionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionScheduleAdjuster {
    pub percent: f64,
}

impl RedactionScheduleModifier for PercentRedactionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionScheduleAdjuster {
    pub minimum: f64,
}

impl RedactionScheduleModifier for MinimumRedactionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the redaction schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionScheduleEngine {
    pub steps: Vec<Rc<dyn RedactionScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionScheduleEngine {
    pub fn run(&mut self, item: &RedactionSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RedactionSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_redaction_schedule_engine() -> RedactionScheduleEngine {
    RedactionScheduleEngine {
        steps: vec![
            Rc::new(UnitRedactionScheduleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.01), ("retail".to_string(), 0.71)]),
            }) as Rc<dyn RedactionScheduleModifier>,
            Rc::new(RedactionScheduleScoreOffsetAdjuster { weight: 1.77 }),
            Rc::new(TieredRedactionScheduleAdjuster {
                bounds: vec![6577.0, 7330.0],
                rates: vec![1.17, 0.98],
            }),
            Rc::new(PercentRedactionScheduleAdjuster { percent: 13.4 }),
            Rc::new(MinimumRedactionScheduleAdjuster { minimum: 2.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 93466.8,
    }
}
