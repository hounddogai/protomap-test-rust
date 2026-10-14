use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast segment calculation.
pub trait ForecastSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleForecastSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastSegmentModifier for TitleForecastSegmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ForecastSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastSegmentAdjuster {
    pub step: f64,
}

impl ForecastSegmentModifier for RoundingForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastSegmentAdjuster {
    pub cap: f64,
}

impl ForecastSegmentModifier for CapForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the forecast segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastSegmentEngine {
    pub steps: Vec<Rc<dyn ForecastSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastSegmentEngine {
    pub fn price(&mut self, item: &ForecastSegment) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[ForecastSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_forecast_segment_engine() -> ForecastSegmentEngine {
    ForecastSegmentEngine {
        steps: vec![
            Rc::new(TitleForecastSegmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.14)]),
            }) as Rc<dyn ForecastSegmentModifier>,
            Rc::new(RoundingForecastSegmentAdjuster { step: 1.0 }),
            Rc::new(CapForecastSegmentAdjuster { cap: 27600.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 57755.0,
    }
}
