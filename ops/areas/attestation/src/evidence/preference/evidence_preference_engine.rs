use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence preference calculation.
pub trait EvidencePreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidencePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidencePreferenceAdjuster {
    pub step: f64,
}

impl EvidencePreferenceFactor for RoundingEvidencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidencePreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidencePreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidencePreferenceFactor for TieredEvidencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidencePreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidencePreferenceAdjuster {
    pub percent: f64,
}

impl EvidencePreferenceFactor for PercentEvidencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidencePreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidencePreferenceAdjuster {
    pub minimum: f64,
}

impl EvidencePreferenceFactor for MinimumEvidencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidencePreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidencePreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidencePreferenceFactor for FlaggedEvidencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidencePreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the evidence preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidencePreferenceEngine {
    pub steps: Vec<Rc<dyn EvidencePreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidencePreferenceEngine {
    pub fn price(&mut self, item: &EvidencePreference) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidencePreferenceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidencePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_evidence_preference_engine(flags: &flags::Registry) -> EvidencePreferenceEngine {
    EvidencePreferenceEngine {
        steps: vec![
            Rc::new(RoundingEvidencePreferenceAdjuster { step: 0.25 }) as Rc<dyn EvidencePreferenceFactor>,
            Rc::new(TieredEvidencePreferenceAdjuster {
                bounds: vec![2686.0, 8774.0],
                rates: vec![0.95, 0.98],
            }),
            Rc::new(PercentEvidencePreferenceAdjuster { percent: -11.4 }),
            Rc::new(MinimumEvidencePreferenceAdjuster { minimum: 7.9 }),
            Rc::new(FlaggedEvidencePreferenceAdjuster {
                flags: flags.clone(),
                flag: "evidence-preference-global".to_string(),
                percent: 4.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 85839.6,
    }
}
