use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert bundle calculation.
pub trait AlertBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertBundleAdjuster {
    pub cap: f64,
}

impl AlertBundleModifier for CapAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertBundleModifier for TieredAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertBundleModifier for FlaggedAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertBundleAdjuster {
    pub percent: f64,
}

impl AlertBundleModifier for PercentAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertBundleAdjuster {
    pub minimum: f64,
}

impl AlertBundleModifier for MinimumAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAlertBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertBundleModifier for ChannelAlertBundleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AlertBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the alert bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertBundleEngine {
    pub steps: Vec<Rc<dyn AlertBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertBundleEngine {
    pub fn run(&mut self, item: &AlertBundle) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[AlertBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_alert_bundle_engine(flags: &flags::Registry) -> AlertBundleEngine {
    AlertBundleEngine {
        steps: vec![
            Rc::new(CapAlertBundleAdjuster { cap: 36639.0 }) as Rc<dyn AlertBundleModifier>,
            Rc::new(TieredAlertBundleAdjuster {
                bounds: vec![426.0, 7137.0, 7994.0],
                rates: vec![1.05, 0.89, 1.18],
            }),
            Rc::new(FlaggedAlertBundleAdjuster {
                flags: flags.clone(),
                flag: "alert-bundle-default".to_string(),
                percent: -3.1,
            }),
            Rc::new(PercentAlertBundleAdjuster { percent: 12.6 }),
            Rc::new(MinimumAlertBundleAdjuster { minimum: 17.0 }),
            Rc::new(ChannelAlertBundleAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.83),
                    ("retail".to_string(), 0.89),
                    ("online".to_string(), 0.81),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 47974.2,
    }
}
