use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline forecast calculation.
pub trait TimelineForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineForecastAdjuster {
    pub step: f64,
}

impl TimelineForecastModifier for RoundingTimelineForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineForecastAdjuster {
    pub cap: f64,
}

impl TimelineForecastModifier for CapTimelineForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceTimelineForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelineForecastModifier for NamespaceTimelineForecastAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &TimelineForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineForecastPriorityOffsetAdjuster {
    pub weight: f64,
}

impl TimelineForecastModifier for TimelineForecastPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineForecast, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the timeline forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineForecastEngine {
    pub steps: Vec<Rc<dyn TimelineForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineForecastEngine {
    pub fn compute(&mut self, item: &TimelineForecast) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[TimelineForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_timeline_forecast_engine() -> TimelineForecastEngine {
    TimelineForecastEngine {
        steps: vec![
            Rc::new(RoundingTimelineForecastAdjuster { step: 0.1 }) as Rc<dyn TimelineForecastModifier>,
            Rc::new(CapTimelineForecastAdjuster { cap: 37704.0 }),
            Rc::new(NamespaceTimelineForecastAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.04), ("south".to_string(), 1.37)]),
            }),
            Rc::new(TimelineForecastPriorityOffsetAdjuster { weight: 0.41 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 31868.8,
    }
}
