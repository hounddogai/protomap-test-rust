use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the page tier calculation.
pub trait PageTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageTierAdjuster {
    pub minimum: f64,
}

impl PageTierAdjuster for MinimumPageTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageTierAdjuster for FlaggedPageTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageTierAdjuster {
    pub cap: f64,
}

impl PageTierAdjuster for CapPageTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the page tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageTierEngine {
    pub steps: Vec<Rc<dyn PageTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageTierEngine {
    pub fn price(&mut self, item: &PageTier) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageTierAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_page_tier_engine(flags: &flags::Registry) -> PageTierEngine {
    PageTierEngine {
        steps: vec![
            Rc::new(MinimumPageTierAdjuster { minimum: 48.1 }) as Rc<dyn PageTierAdjuster>,
            Rc::new(FlaggedPageTierAdjuster {
                flags: flags.clone(),
                flag: "page-tier-scheduled".to_string(),
                percent: -5.1,
            }),
            Rc::new(CapPageTierAdjuster { cap: 37168.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 42972.8,
    }
}
