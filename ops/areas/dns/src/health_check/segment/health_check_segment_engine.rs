use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the health check segment calculation.
pub trait HealthCheckSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckSegmentAdjuster {
    pub percent: f64,
}

impl HealthCheckSegmentStep for PercentHealthCheckSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckSegmentAdjuster {
    pub step: f64,
}

impl HealthCheckSegmentStep for RoundingHealthCheckSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryHealthCheckSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckSegmentStep for CountryHealthCheckSegmentAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &HealthCheckSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the health check segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckSegmentEngine {
    pub steps: Vec<Rc<dyn HealthCheckSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckSegmentEngine {
    pub fn compute(&mut self, item: &HealthCheckSegment) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_health_check_segment_engine() -> HealthCheckSegmentEngine {
    HealthCheckSegmentEngine {
        steps: vec![
            Rc::new(PercentHealthCheckSegmentAdjuster { percent: 2.3 }) as Rc<dyn HealthCheckSegmentStep>,
            Rc::new(RoundingHealthCheckSegmentAdjuster { step: 5.0 }),
            Rc::new(CountryHealthCheckSegmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.73)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 90448.5,
    }
}
