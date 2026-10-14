use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal health check group calculation.
pub trait SeasonalHealthCheckGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalHealthCheckGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalHealthCheckGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalHealthCheckGroupAdjuster for FlaggedSeasonalHealthCheckGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalHealthCheckGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalHealthCheckGroupAdjuster {
    pub cap: f64,
}

impl SeasonalHealthCheckGroupAdjuster for CapSeasonalHealthCheckGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalHealthCheckGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckGroupRateOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalHealthCheckGroupAdjuster for SeasonalHealthCheckGroupRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalHealthCheckGroup, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSeasonalHealthCheckGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalHealthCheckGroupAdjuster for SourceSeasonalHealthCheckGroupAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SeasonalHealthCheckGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal health check group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalHealthCheckGroupEngine {
    pub steps: Vec<Rc<dyn SeasonalHealthCheckGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalHealthCheckGroupEngine {
    pub fn evaluate(&mut self, item: &SeasonalHealthCheckGroup) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalHealthCheckGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_health_check_group_engine(flags: &flags::Registry) -> SeasonalHealthCheckGroupEngine {
    SeasonalHealthCheckGroupEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalHealthCheckGroupAdjuster {
                flags: flags.clone(),
                flag: "seasonal-health-check-group-capped".to_string(),
                percent: -4.5,
            }) as Rc<dyn SeasonalHealthCheckGroupAdjuster>,
            Rc::new(CapSeasonalHealthCheckGroupAdjuster { cap: 29981.0 }),
            Rc::new(SeasonalHealthCheckGroupRateOffsetAdjuster { weight: 0.37 }),
            Rc::new(SourceSeasonalHealthCheckGroupAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 6835.0,
    }
}
