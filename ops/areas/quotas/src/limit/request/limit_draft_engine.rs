use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit draft calculation.
pub trait LimitDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitDraftAdjuster {
    pub percent: f64,
}

impl LimitDraftStep for PercentLimitDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitDraftAdjuster {
    pub step: f64,
}

impl LimitDraftStep for RoundingLimitDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitDraftAdjuster {
    pub minimum: f64,
}

impl LimitDraftStep for MinimumLimitDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitDraftStep for FlaggedLimitDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDraftWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl LimitDraftStep for LimitDraftWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &LimitDraft, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the limit draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitDraftEngine {
    pub steps: Vec<Rc<dyn LimitDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitDraftEngine {
    pub fn price(&mut self, item: &LimitDraft) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_limit_draft_engine(flags: &flags::Registry) -> LimitDraftEngine {
    LimitDraftEngine {
        steps: vec![
            Rc::new(PercentLimitDraftAdjuster { percent: -2.8 }) as Rc<dyn LimitDraftStep>,
            Rc::new(RoundingLimitDraftAdjuster { step: 0.1 }),
            Rc::new(MinimumLimitDraftAdjuster { minimum: 22.7 }),
            Rc::new(FlaggedLimitDraftAdjuster {
                flags: flags.clone(),
                flag: "limit-draft-pending".to_string(),
                percent: 4.3,
            }),
            Rc::new(LimitDraftWindowDaysOffsetAdjuster { weight: 0.63 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 92236.4,
    }
}
