use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive page variance calculation.
pub trait AdaptivePageVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptivePageVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageVarianceRevisionOffsetAdjuster {
    pub weight: f64,
}

impl AdaptivePageVarianceStep for AdaptivePageVarianceRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptivePageVariance, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptivePageVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptivePageVarianceStep for TieredAdaptivePageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptivePageVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptivePageVarianceAdjuster {
    pub minimum: f64,
}

impl AdaptivePageVarianceStep for MinimumAdaptivePageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptivePageVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAdaptivePageVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptivePageVarianceStep for UnitAdaptivePageVarianceAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &AdaptivePageVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptivePageVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptivePageVarianceStep for FlaggedAdaptivePageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptivePageVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive page variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptivePageVarianceEngine {
    pub steps: Vec<Rc<dyn AdaptivePageVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptivePageVarianceEngine {
    pub fn run(&mut self, item: &AdaptivePageVariance) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptivePageVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_page_variance_engine(flags: &flags::Registry) -> AdaptivePageVarianceEngine {
    AdaptivePageVarianceEngine {
        steps: vec![
            Rc::new(AdaptivePageVarianceRevisionOffsetAdjuster { weight: 0.91 }) as Rc<dyn AdaptivePageVarianceStep>,
            Rc::new(TieredAdaptivePageVarianceAdjuster {
                bounds: vec![731.0, 912.0],
                rates: vec![0.84, 1.17],
            }),
            Rc::new(MinimumAdaptivePageVarianceAdjuster { minimum: 12.2 }),
            Rc::new(UnitAdaptivePageVarianceAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.7),
                    ("south".to_string(), 0.82),
                    ("retail".to_string(), 0.94),
                ]),
            }),
            Rc::new(FlaggedAdaptivePageVarianceAdjuster {
                flags: flags.clone(),
                flag: "adaptive-page-variance-default".to_string(),
                percent: -6.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 22743.3,
    }
}
