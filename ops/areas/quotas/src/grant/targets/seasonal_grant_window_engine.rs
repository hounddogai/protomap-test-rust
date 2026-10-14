use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal grant window calculation.
pub trait SeasonalGrantWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalGrantWindow, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalGrantWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalGrantWindowStep for FlaggedSeasonalGrantWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalGrantWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalGrantWindowAdjuster {
    pub cap: f64,
}

impl SeasonalGrantWindowStep for CapSeasonalGrantWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalGrantWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalGrantWindowAdjuster {
    pub step: f64,
}

impl SeasonalGrantWindowStep for RoundingSeasonalGrantWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalGrantWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalGrantWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalGrantWindowStep for TieredSeasonalGrantWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalGrantWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantWindowPriorityOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalGrantWindowStep for SeasonalGrantWindowPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalGrantWindow, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the seasonal grant window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalGrantWindowEngine {
    pub steps: Vec<Rc<dyn SeasonalGrantWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalGrantWindowEngine {
    pub fn run(&mut self, item: &SeasonalGrantWindow) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalGrantWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_grant_window_engine(flags: &flags::Registry) -> SeasonalGrantWindowEngine {
    SeasonalGrantWindowEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalGrantWindowAdjuster {
                flags: flags.clone(),
                flag: "seasonal-grant-window-seasonal".to_string(),
                percent: 10.0,
            }) as Rc<dyn SeasonalGrantWindowStep>,
            Rc::new(CapSeasonalGrantWindowAdjuster { cap: 47547.0 }),
            Rc::new(RoundingSeasonalGrantWindowAdjuster { step: 0.1 }),
            Rc::new(TieredSeasonalGrantWindowAdjuster {
                bounds: vec![6687.0, 6753.0, 7954.0],
                rates: vec![1.15, 1.09, 0.89],
            }),
            Rc::new(SeasonalGrantWindowPriorityOffsetAdjuster { weight: 1.74 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 54063.7,
    }
}
