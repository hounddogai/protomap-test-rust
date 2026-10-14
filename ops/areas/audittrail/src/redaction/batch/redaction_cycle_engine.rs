use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction cycle calculation.
pub trait RedactionCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionCycleAdjuster {
    pub percent: f64,
}

impl RedactionCycleAdjuster for PercentRedactionCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionCycleAdjuster for TieredRedactionCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionCycleAdjuster {
    pub cap: f64,
}

impl RedactionCycleAdjuster for CapRedactionCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionCycleAdjuster {
    pub step: f64,
}

impl RedactionCycleAdjuster for RoundingRedactionCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionCycleAdjuster for FlaggedRedactionCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRedactionCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionCycleAdjuster for TitleRedactionCycleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RedactionCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the redaction cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionCycleEngine {
    pub steps: Vec<Rc<dyn RedactionCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionCycleEngine {
    pub fn price(&mut self, item: &RedactionCycle) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[RedactionCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_cycle_engine(flags: &flags::Registry) -> RedactionCycleEngine {
    RedactionCycleEngine {
        steps: vec![
            Rc::new(PercentRedactionCycleAdjuster { percent: -14.8 }) as Rc<dyn RedactionCycleAdjuster>,
            Rc::new(TieredRedactionCycleAdjuster {
                bounds: vec![6977.0, 8767.0],
                rates: vec![1.13, 1.07],
            }),
            Rc::new(CapRedactionCycleAdjuster { cap: 3687.0 }),
            Rc::new(RoundingRedactionCycleAdjuster { step: 5.0 }),
            Rc::new(FlaggedRedactionCycleAdjuster {
                flags: flags.clone(),
                flag: "redaction-cycle-scheduled".to_string(),
                percent: -3.0,
            }),
            Rc::new(TitleRedactionCycleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.04), ("west".to_string(), 0.89)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 74397.3,
    }
}
