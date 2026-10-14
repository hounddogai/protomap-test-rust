use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional page draft calculation.
pub trait RegionalPageDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPageDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalPageDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalPageDraftStep for FlaggedRegionalPageDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalPageDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalPageDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalPageDraftStep for TieredRegionalPageDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalPageDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraftWeightOffsetAdjuster {
    pub weight: f64,
}

impl RegionalPageDraftStep for RegionalPageDraftWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalPageDraft, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionalPageDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalPageDraftStep for TitleRegionalPageDraftAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionalPageDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional page draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPageDraftEngine {
    pub steps: Vec<Rc<dyn RegionalPageDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPageDraftEngine {
    pub fn evaluate(&mut self, item: &RegionalPageDraft) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalPageDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPageDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_page_draft_engine(flags: &flags::Registry) -> RegionalPageDraftEngine {
    RegionalPageDraftEngine {
        steps: vec![
            Rc::new(FlaggedRegionalPageDraftAdjuster {
                flags: flags.clone(),
                flag: "regional-page-draft-primary".to_string(),
                percent: -7.4,
            }) as Rc<dyn RegionalPageDraftStep>,
            Rc::new(TieredRegionalPageDraftAdjuster {
                bounds: vec![555.0, 7278.0, 8368.0],
                rates: vec![1.08, 0.9, 0.96],
            }),
            Rc::new(RegionalPageDraftWeightOffsetAdjuster { weight: 0.83 }),
            Rc::new(TitleRegionalPageDraftAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.77), ("south".to_string(), 0.89)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 78303.0,
    }
}
