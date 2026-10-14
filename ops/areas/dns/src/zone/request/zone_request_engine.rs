use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone request calculation.
pub trait ZoneRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequestAmountOffsetAdjuster {
    pub weight: f64,
}

impl ZoneRequestStep for ZoneRequestAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneRequest, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneRequestAdjuster {
    pub cap: f64,
}

impl ZoneRequestStep for CapZoneRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneRequestAdjuster {
    pub step: f64,
}

impl ZoneRequestStep for RoundingZoneRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceZoneRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneRequestStep for SourceZoneRequestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ZoneRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the zone request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneRequestEngine {
    pub steps: Vec<Rc<dyn ZoneRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneRequestEngine {
    pub fn evaluate(&mut self, item: &ZoneRequest) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneRequestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_zone_request_engine() -> ZoneRequestEngine {
    ZoneRequestEngine {
        steps: vec![
            Rc::new(ZoneRequestAmountOffsetAdjuster { weight: 1.22 }) as Rc<dyn ZoneRequestStep>,
            Rc::new(CapZoneRequestAdjuster { cap: 45339.0 }),
            Rc::new(RoundingZoneRequestAdjuster { step: 0.5 }),
            Rc::new(SourceZoneRequestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.75)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 99269.1,
    }
}
