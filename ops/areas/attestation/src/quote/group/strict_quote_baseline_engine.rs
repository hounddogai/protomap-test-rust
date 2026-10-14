use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict quote baseline calculation.
pub trait StrictQuoteBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictQuoteBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictQuoteBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictQuoteBaselineFactor for FlaggedStrictQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictQuoteBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictQuoteBaselineAdjuster {
    pub cap: f64,
}

impl StrictQuoteBaselineFactor for CapStrictQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictQuoteBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleStrictQuoteBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictQuoteBaselineFactor for TitleStrictQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &StrictQuoteBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictQuoteBaselineAdjuster {
    pub minimum: f64,
}

impl StrictQuoteBaselineFactor for MinimumStrictQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictQuoteBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict quote baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictQuoteBaselineEngine {
    pub steps: Vec<Rc<dyn StrictQuoteBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictQuoteBaselineEngine {
    pub fn run(&mut self, item: &StrictQuoteBaseline) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictQuoteBaselineFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictQuoteBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_quote_baseline_engine(flags: &flags::Registry) -> StrictQuoteBaselineEngine {
    StrictQuoteBaselineEngine {
        steps: vec![
            Rc::new(FlaggedStrictQuoteBaselineAdjuster {
                flags: flags.clone(),
                flag: "strict-quote-baseline-default".to_string(),
                percent: -0.3,
            }) as Rc<dyn StrictQuoteBaselineFactor>,
            Rc::new(CapStrictQuoteBaselineAdjuster { cap: 23053.0 }),
            Rc::new(TitleStrictQuoteBaselineAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.35)]),
            }),
            Rc::new(MinimumStrictQuoteBaselineAdjuster { minimum: 47.4 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 2402.4,
    }
}
