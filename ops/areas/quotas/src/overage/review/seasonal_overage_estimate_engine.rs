use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal overage estimate calculation.
pub trait SeasonalOverageEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalOverageEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageEstimateRateOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalOverageEstimateAdjuster for SeasonalOverageEstimateRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalOverageEstimate, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalOverageEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalOverageEstimateAdjuster for FlaggedSeasonalOverageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalOverageEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalOverageEstimateAdjuster {
    pub minimum: f64,
}

impl SeasonalOverageEstimateAdjuster for MinimumSeasonalOverageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalOverageEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalOverageEstimateAdjuster {
    pub cap: f64,
}

impl SeasonalOverageEstimateAdjuster for CapSeasonalOverageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalOverageEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalOverageEstimateAdjuster {
    pub step: f64,
}

impl SeasonalOverageEstimateAdjuster for RoundingSeasonalOverageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalOverageEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeSeasonalOverageEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalOverageEstimateAdjuster for ScopeSeasonalOverageEstimateAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &SeasonalOverageEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal overage estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalOverageEstimateEngine {
    pub steps: Vec<Rc<dyn SeasonalOverageEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalOverageEstimateEngine {
    pub fn evaluate(&mut self, item: &SeasonalOverageEstimate) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalOverageEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_overage_estimate_engine(flags: &flags::Registry) -> SeasonalOverageEstimateEngine {
    SeasonalOverageEstimateEngine {
        steps: vec![
            Rc::new(SeasonalOverageEstimateRateOffsetAdjuster { weight: 1.97 })
                as Rc<dyn SeasonalOverageEstimateAdjuster>,
            Rc::new(FlaggedSeasonalOverageEstimateAdjuster {
                flags: flags.clone(),
                flag: "seasonal-overage-estimate-pending".to_string(),
                percent: -4.3,
            }),
            Rc::new(MinimumSeasonalOverageEstimateAdjuster { minimum: 40.9 }),
            Rc::new(CapSeasonalOverageEstimateAdjuster { cap: 11582.0 }),
            Rc::new(RoundingSeasonalOverageEstimateAdjuster { step: 5.0 }),
            Rc::new(ScopeSeasonalOverageEstimateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.22)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 79215.3,
    }
}
