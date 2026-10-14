use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the query schedule calculation.
pub trait QueryScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuerySchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelQueryScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryScheduleAdjuster for LabelQueryScheduleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &QuerySchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryScheduleAdjuster {
    pub minimum: f64,
}

impl QueryScheduleAdjuster for MinimumQueryScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuerySchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryScheduleAdjuster {
    pub cap: f64,
}

impl QueryScheduleAdjuster for CapQueryScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuerySchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryScheduleAdjuster {
    pub percent: f64,
}

impl QueryScheduleAdjuster for PercentQueryScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuerySchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryScheduleEngine {
    pub steps: Vec<Rc<dyn QueryScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryScheduleEngine {
    pub fn run(&mut self, item: &QuerySchedule) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryScheduleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuerySchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_query_schedule_engine() -> QueryScheduleEngine {
    QueryScheduleEngine {
        steps: vec![
            Rc::new(LabelQueryScheduleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.77),
                    ("east".to_string(), 1.18),
                    ("north".to_string(), 1.25),
                ]),
            }) as Rc<dyn QueryScheduleAdjuster>,
            Rc::new(MinimumQueryScheduleAdjuster { minimum: 39.0 }),
            Rc::new(CapQueryScheduleAdjuster { cap: 30399.0 }),
            Rc::new(PercentQueryScheduleAdjuster { percent: -1.8 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 68454.5,
    }
}
