use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page group calculation.
pub trait PageGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageGroupStep for TieredPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageGroupAdjuster {
    pub percent: f64,
}

impl PageGroupStep for PercentPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageGroupAdjuster {
    pub step: f64,
}

impl PageGroupStep for RoundingPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageGroupStep for FlaggedPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageGroupEngine {
    pub steps: Vec<Rc<dyn PageGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageGroupEngine {
    pub fn compute(&mut self, item: &PageGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PageGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_page_group_engine(flags: &flags::Registry) -> PageGroupEngine {
    PageGroupEngine {
        steps: vec![
            Rc::new(TieredPageGroupAdjuster {
                bounds: vec![1588.0, 1894.0, 2719.0],
                rates: vec![1.01, 0.83, 1.06],
            }) as Rc<dyn PageGroupStep>,
            Rc::new(PercentPageGroupAdjuster { percent: 7.0 }),
            Rc::new(RoundingPageGroupAdjuster { step: 0.5 }),
            Rc::new(FlaggedPageGroupAdjuster {
                flags: flags.clone(),
                flag: "page-group-scheduled".to_string(),
                percent: 9.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 66535.0,
    }
}
