use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy quote checkpoint calculation.
pub trait LegacyQuoteCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyQuoteCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyQuoteCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyQuoteCheckpointStep for FlaggedLegacyQuoteCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyQuoteCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteCheckpointPriorityOffsetAdjuster {
    pub weight: f64,
}

impl LegacyQuoteCheckpointStep for LegacyQuoteCheckpointPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyQuoteCheckpoint, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLegacyQuoteCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyQuoteCheckpointStep for RegionLegacyQuoteCheckpointAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &LegacyQuoteCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyQuoteCheckpointAdjuster {
    pub cap: f64,
}

impl LegacyQuoteCheckpointStep for CapLegacyQuoteCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyQuoteCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyQuoteCheckpointAdjuster {
    pub step: f64,
}

impl LegacyQuoteCheckpointStep for RoundingLegacyQuoteCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyQuoteCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy quote checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyQuoteCheckpointEngine {
    pub steps: Vec<Rc<dyn LegacyQuoteCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyQuoteCheckpointEngine {
    pub fn evaluate(&mut self, item: &LegacyQuoteCheckpoint) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyQuoteCheckpointStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyQuoteCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_quote_checkpoint_engine(flags: &flags::Registry) -> LegacyQuoteCheckpointEngine {
    LegacyQuoteCheckpointEngine {
        steps: vec![
            Rc::new(FlaggedLegacyQuoteCheckpointAdjuster {
                flags: flags.clone(),
                flag: "legacy-quote-checkpoint-legacy".to_string(),
                percent: -1.1,
            }) as Rc<dyn LegacyQuoteCheckpointStep>,
            Rc::new(LegacyQuoteCheckpointPriorityOffsetAdjuster { weight: 0.48 }),
            Rc::new(RegionLegacyQuoteCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.72),
                    ("retail".to_string(), 0.96),
                    ("west".to_string(), 0.76),
                ]),
            }),
            Rc::new(CapLegacyQuoteCheckpointAdjuster { cap: 48280.0 }),
            Rc::new(RoundingLegacyQuoteCheckpointAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 24116.7,
    }
}
