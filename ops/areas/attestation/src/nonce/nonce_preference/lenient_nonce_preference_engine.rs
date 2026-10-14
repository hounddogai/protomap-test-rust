use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient nonce preference calculation.
pub trait LenientNoncePreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientNoncePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientNoncePreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientNoncePreferenceStep for TieredLenientNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientNoncePreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLenientNoncePreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientNoncePreferenceStep for SourceLenientNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &LenientNoncePreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientNoncePreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientNoncePreferenceStep for FlaggedLenientNoncePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientNoncePreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient nonce preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientNoncePreferenceEngine {
    pub steps: Vec<Rc<dyn LenientNoncePreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientNoncePreferenceEngine {
    pub fn run(&mut self, item: &LenientNoncePreference) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientNoncePreferenceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientNoncePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_nonce_preference_engine(flags: &flags::Registry) -> LenientNoncePreferenceEngine {
    LenientNoncePreferenceEngine {
        steps: vec![
            Rc::new(TieredLenientNoncePreferenceAdjuster {
                bounds: vec![5820.0, 7117.0],
                rates: vec![1.04, 0.99],
            }) as Rc<dyn LenientNoncePreferenceStep>,
            Rc::new(SourceLenientNoncePreferenceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.02), ("central".to_string(), 1.24)]),
            }),
            Rc::new(FlaggedLenientNoncePreferenceAdjuster {
                flags: flags.clone(),
                flag: "lenient-nonce-preference-scheduled".to_string(),
                percent: 5.1,
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 45115.5,
    }
}
