use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier preference calculation.
pub trait VerifierPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelVerifierPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierPreferenceStep for LabelVerifierPreferenceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &VerifierPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierPreferenceAdjuster {
    pub step: f64,
}

impl VerifierPreferenceStep for RoundingVerifierPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierPreferenceAdjuster {
    pub cap: f64,
}

impl VerifierPreferenceStep for CapVerifierPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPreferenceBurstOffsetAdjuster {
    pub weight: f64,
}

impl VerifierPreferenceStep for VerifierPreferenceBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierPreference, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierPreferenceStep for FlaggedVerifierPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the verifier preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierPreferenceEngine {
    pub steps: Vec<Rc<dyn VerifierPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierPreferenceEngine {
    pub fn price(&mut self, item: &VerifierPreference) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[VerifierPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_verifier_preference_engine(flags: &flags::Registry) -> VerifierPreferenceEngine {
    VerifierPreferenceEngine {
        steps: vec![
            Rc::new(LabelVerifierPreferenceAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.17)]),
            }) as Rc<dyn VerifierPreferenceStep>,
            Rc::new(RoundingVerifierPreferenceAdjuster { step: 0.5 }),
            Rc::new(CapVerifierPreferenceAdjuster { cap: 46202.0 }),
            Rc::new(VerifierPreferenceBurstOffsetAdjuster { weight: 1.26 }),
            Rc::new(FlaggedVerifierPreferenceAdjuster {
                flags: flags.clone(),
                flag: "verifier-preference-fallback".to_string(),
                percent: 2.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 11599.6,
    }
}
