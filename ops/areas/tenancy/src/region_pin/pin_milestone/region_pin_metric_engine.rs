use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the region pin metric calculation.
pub trait RegionPinMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinMetricAdjuster {
    pub minimum: f64,
}

impl RegionPinMetricAdjuster for MinimumRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinMetricAdjuster for FlaggedRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRegionPinMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinMetricAdjuster for UnitRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RegionPinMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the region pin metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinMetricEngine {
    pub steps: Vec<Rc<dyn RegionPinMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinMetricEngine {
    pub fn run(&mut self, item: &RegionPinMetric) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_region_pin_metric_engine(flags: &flags::Registry) -> RegionPinMetricEngine {
    RegionPinMetricEngine {
        steps: vec![
            Rc::new(MinimumRegionPinMetricAdjuster { minimum: 41.2 }) as Rc<dyn RegionPinMetricAdjuster>,
            Rc::new(FlaggedRegionPinMetricAdjuster {
                flags: flags.clone(),
                flag: "region-pin-metric-weighted".to_string(),
                percent: 2.3,
            }),
            Rc::new(UnitRegionPinMetricAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.97)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 51814.3,
    }
}
