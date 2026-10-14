use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook draft calculation.
pub trait RunbookDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRunbookDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookDraftModifier for CountryRunbookDraftAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RunbookDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookDraftAdjuster {
    pub percent: f64,
}

impl RunbookDraftModifier for PercentRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookDraftAdjuster {
    pub minimum: f64,
}

impl RunbookDraftModifier for MinimumRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDraftQuantityOffsetAdjuster {
    pub weight: f64,
}

impl RunbookDraftModifier for RunbookDraftQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookDraft, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookDraftModifier for FlaggedRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookDraftEngine {
    pub steps: Vec<Rc<dyn RunbookDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookDraftEngine {
    pub fn price(&mut self, item: &RunbookDraft) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RunbookDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_runbook_draft_engine(flags: &flags::Registry) -> RunbookDraftEngine {
    RunbookDraftEngine {
        steps: vec![
            Rc::new(CountryRunbookDraftAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.87),
                    ("west".to_string(), 1.17),
                    ("south".to_string(), 1.25),
                ]),
            }) as Rc<dyn RunbookDraftModifier>,
            Rc::new(PercentRunbookDraftAdjuster { percent: -6.9 }),
            Rc::new(MinimumRunbookDraftAdjuster { minimum: 11.5 }),
            Rc::new(RunbookDraftQuantityOffsetAdjuster { weight: 0.15 }),
            Rc::new(FlaggedRunbookDraftAdjuster {
                flags: flags.clone(),
                flag: "runbook-draft-default".to_string(),
                percent: -3.8,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 7673.2,
    }
}
