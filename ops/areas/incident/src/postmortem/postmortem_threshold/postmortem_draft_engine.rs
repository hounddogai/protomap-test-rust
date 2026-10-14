use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the postmortem draft calculation.
pub trait PostmortemDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDraftCapacityOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemDraftStep for PostmortemDraftCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemDraft, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPostmortemDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemDraftStep for LabelPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PostmortemDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemDraftStep for FlaggedPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemDraftStep for TieredPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemDraftAdjuster {
    pub minimum: f64,
}

impl PostmortemDraftStep for MinimumPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemDraftAdjuster {
    pub cap: f64,
}

impl PostmortemDraftStep for CapPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the postmortem draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemDraftEngine {
    pub steps: Vec<Rc<dyn PostmortemDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemDraftEngine {
    pub fn price(&mut self, item: &PostmortemDraft) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_draft_engine(flags: &flags::Registry) -> PostmortemDraftEngine {
    PostmortemDraftEngine {
        steps: vec![
            Rc::new(PostmortemDraftCapacityOffsetAdjuster { weight: 0.94 }) as Rc<dyn PostmortemDraftStep>,
            Rc::new(LabelPostmortemDraftAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.2), ("north".to_string(), 0.78)]),
            }),
            Rc::new(FlaggedPostmortemDraftAdjuster {
                flags: flags.clone(),
                flag: "postmortem-draft-seasonal".to_string(),
                percent: -7.2,
            }),
            Rc::new(TieredPostmortemDraftAdjuster {
                bounds: vec![2797.0, 4530.0, 4741.0, 9795.0],
                rates: vec![1.14, 1.03, 1.0, 0.95],
            }),
            Rc::new(MinimumPostmortemDraftAdjuster { minimum: 32.2 }),
            Rc::new(CapPostmortemDraftAdjuster { cap: 46533.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 84175.8,
    }
}
