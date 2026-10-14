use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional egress exception calculation.
pub trait RegionalEgressExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalEgressException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalEgressExceptionAdjuster {
    pub percent: f64,
}

impl RegionalEgressExceptionStep for PercentRegionalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalEgressException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalEgressExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalEgressExceptionStep for FlaggedRegionalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalEgressException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalEgressExceptionAdjuster {
    pub minimum: f64,
}

impl RegionalEgressExceptionStep for MinimumRegionalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalEgressException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalEgressExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalEgressExceptionStep for TieredRegionalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalEgressException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalEgressExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalEgressExceptionStep for NameRegionalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalEgressException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressExceptionSequenceOffsetAdjuster {
    pub weight: f64,
}

impl RegionalEgressExceptionStep for RegionalEgressExceptionSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalEgressException, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the regional egress exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalEgressExceptionEngine {
    pub steps: Vec<Rc<dyn RegionalEgressExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalEgressExceptionEngine {
    pub fn evaluate(&mut self, item: &RegionalEgressException) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalEgressExceptionStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalEgressException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_egress_exception_engine(flags: &flags::Registry) -> RegionalEgressExceptionEngine {
    RegionalEgressExceptionEngine {
        steps: vec![
            Rc::new(PercentRegionalEgressExceptionAdjuster { percent: 20.9 }) as Rc<dyn RegionalEgressExceptionStep>,
            Rc::new(FlaggedRegionalEgressExceptionAdjuster {
                flags: flags.clone(),
                flag: "regional-egress-exception-adaptive".to_string(),
                percent: -5.2,
            }),
            Rc::new(MinimumRegionalEgressExceptionAdjuster { minimum: 15.0 }),
            Rc::new(TieredRegionalEgressExceptionAdjuster {
                bounds: vec![838.0, 8295.0],
                rates: vec![0.98, 0.96],
            }),
            Rc::new(NameRegionalEgressExceptionAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.19), ("west".to_string(), 0.98)]),
            }),
            Rc::new(RegionalEgressExceptionSequenceOffsetAdjuster { weight: 1.62 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 7571.4,
    }
}
