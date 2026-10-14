use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient postmortem draft calculation.
pub trait LenientPostmortemDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPostmortemDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientPostmortemDraftAdjuster {
    pub cap: f64,
}

impl LenientPostmortemDraftStep for CapLenientPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientPostmortemDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientPostmortemDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientPostmortemDraftStep for FlaggedLenientPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientPostmortemDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientPostmortemDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientPostmortemDraftStep for TieredLenientPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientPostmortemDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientPostmortemDraftAdjuster {
    pub step: f64,
}

impl LenientPostmortemDraftStep for RoundingLenientPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientPostmortemDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelLenientPostmortemDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientPostmortemDraftStep for ChannelLenientPostmortemDraftAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &LenientPostmortemDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient postmortem draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPostmortemDraftEngine {
    pub steps: Vec<Rc<dyn LenientPostmortemDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPostmortemDraftEngine {
    pub fn run(&mut self, item: &LenientPostmortemDraft) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientPostmortemDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientPostmortemDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_postmortem_draft_engine(flags: &flags::Registry) -> LenientPostmortemDraftEngine {
    LenientPostmortemDraftEngine {
        steps: vec![
            Rc::new(CapLenientPostmortemDraftAdjuster { cap: 22292.0 }) as Rc<dyn LenientPostmortemDraftStep>,
            Rc::new(FlaggedLenientPostmortemDraftAdjuster {
                flags: flags.clone(),
                flag: "lenient-postmortem-draft-batched".to_string(),
                percent: 5.2,
            }),
            Rc::new(TieredLenientPostmortemDraftAdjuster {
                bounds: vec![2116.0, 2122.0, 3623.0],
                rates: vec![0.86, 0.83, 1.02],
            }),
            Rc::new(RoundingLenientPostmortemDraftAdjuster { step: 0.05 }),
            Rc::new(ChannelLenientPostmortemDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.25)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 72975.9,
    }
}
