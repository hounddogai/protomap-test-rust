use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem summary calculation.
pub trait PostmortemSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemSummaryFactor for FlaggedPostmortemSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemSummaryAdjuster {
    pub step: f64,
}

impl PostmortemSummaryFactor for RoundingPostmortemSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemSummaryAdjuster {
    pub minimum: f64,
}

impl PostmortemSummaryFactor for MinimumPostmortemSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemSummaryAdjuster {
    pub percent: f64,
}

impl PostmortemSummaryFactor for PercentPostmortemSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemSummaryFactor for TieredPostmortemSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the postmortem summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemSummaryEngine {
    pub steps: Vec<Rc<dyn PostmortemSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemSummaryEngine {
    pub fn price(&mut self, item: &PostmortemSummary) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemSummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_summary_engine(flags: &flags::Registry) -> PostmortemSummaryEngine {
    PostmortemSummaryEngine {
        steps: vec![
            Rc::new(FlaggedPostmortemSummaryAdjuster {
                flags: flags.clone(),
                flag: "postmortem-summary-primary".to_string(),
                percent: -3.9,
            }) as Rc<dyn PostmortemSummaryFactor>,
            Rc::new(RoundingPostmortemSummaryAdjuster { step: 0.1 }),
            Rc::new(MinimumPostmortemSummaryAdjuster { minimum: 27.8 }),
            Rc::new(PercentPostmortemSummaryAdjuster { percent: -9.7 }),
            Rc::new(TieredPostmortemSummaryAdjuster {
                bounds: vec![979.0, 2610.0, 6984.0, 9621.0],
                rates: vec![0.97, 1.13, 1.12, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 17413.0,
    }
}
