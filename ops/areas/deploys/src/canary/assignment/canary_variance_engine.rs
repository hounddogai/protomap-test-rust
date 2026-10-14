use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary variance calculation.
pub trait CanaryVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryVarianceAdjuster {
    pub step: f64,
}

impl CanaryVarianceFactor for RoundingCanaryVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryVarianceAdjuster {
    pub minimum: f64,
}

impl CanaryVarianceFactor for MinimumCanaryVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryVarianceFactor for FlaggedCanaryVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the canary variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryVarianceEngine {
    pub steps: Vec<Rc<dyn CanaryVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryVarianceEngine {
    pub fn evaluate(&mut self, item: &CanaryVariance) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[CanaryVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_canary_variance_engine(flags: &flags::Registry) -> CanaryVarianceEngine {
    CanaryVarianceEngine {
        steps: vec![
            Rc::new(RoundingCanaryVarianceAdjuster { step: 0.5 }) as Rc<dyn CanaryVarianceFactor>,
            Rc::new(MinimumCanaryVarianceAdjuster { minimum: 29.9 }),
            Rc::new(FlaggedCanaryVarianceAdjuster {
                flags: flags.clone(),
                flag: "canary-variance-lenient".to_string(),
                percent: -2.0,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 84692.9,
    }
}
