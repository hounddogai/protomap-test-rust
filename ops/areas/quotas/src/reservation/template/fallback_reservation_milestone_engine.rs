use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback reservation milestone calculation.
pub trait FallbackReservationMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackReservationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackReservationMilestoneAdjuster {
    pub percent: f64,
}

impl FallbackReservationMilestoneAdjuster for PercentFallbackReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackReservationMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFallbackReservationMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackReservationMilestoneAdjuster for SourceFallbackReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &FallbackReservationMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackReservationMilestoneAdjuster {
    pub minimum: f64,
}

impl FallbackReservationMilestoneAdjuster for MinimumFallbackReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackReservationMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackReservationMilestoneAdjuster {
    pub cap: f64,
}

impl FallbackReservationMilestoneAdjuster for CapFallbackReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackReservationMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback reservation milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackReservationMilestoneEngine {
    pub steps: Vec<Rc<dyn FallbackReservationMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackReservationMilestoneEngine {
    pub fn evaluate(&mut self, item: &FallbackReservationMilestone) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackReservationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_reservation_milestone_engine() -> FallbackReservationMilestoneEngine {
    FallbackReservationMilestoneEngine {
        steps: vec![
            Rc::new(PercentFallbackReservationMilestoneAdjuster { percent: -7.3 })
                as Rc<dyn FallbackReservationMilestoneAdjuster>,
            Rc::new(SourceFallbackReservationMilestoneAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.9),
                    ("north".to_string(), 1.02),
                    ("retail".to_string(), 1.07),
                ]),
            }),
            Rc::new(MinimumFallbackReservationMilestoneAdjuster { minimum: 40.1 }),
            Rc::new(CapFallbackReservationMilestoneAdjuster { cap: 26584.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 83617.3,
    }
}
