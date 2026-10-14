use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional egress override calculation.
pub trait RegionalEgressOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalEgressOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalEgressOverrideAdjuster {
    pub percent: f64,
}

impl RegionalEgressOverrideStep for PercentRegionalEgressOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalEgressOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalEgressOverrideAdjuster {
    pub minimum: f64,
}

impl RegionalEgressOverrideStep for MinimumRegionalEgressOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalEgressOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalEgressOverrideAdjuster {
    pub cap: f64,
}

impl RegionalEgressOverrideStep for CapRegionalEgressOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalEgressOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionalEgressOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalEgressOverrideStep for TitleRegionalEgressOverrideAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionalEgressOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional egress override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalEgressOverrideEngine {
    pub steps: Vec<Rc<dyn RegionalEgressOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalEgressOverrideEngine {
    pub fn price(&mut self, item: &RegionalEgressOverride) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalEgressOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_egress_override_engine() -> RegionalEgressOverrideEngine {
    RegionalEgressOverrideEngine {
        steps: vec![
            Rc::new(PercentRegionalEgressOverrideAdjuster { percent: 21.0 }) as Rc<dyn RegionalEgressOverrideStep>,
            Rc::new(MinimumRegionalEgressOverrideAdjuster { minimum: 40.4 }),
            Rc::new(CapRegionalEgressOverrideAdjuster { cap: 17787.0 }),
            Rc::new(TitleRegionalEgressOverrideAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.27), ("central".to_string(), 0.86)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 18468.6,
    }
}
