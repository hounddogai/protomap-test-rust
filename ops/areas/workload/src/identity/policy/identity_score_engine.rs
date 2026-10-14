use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity score calculation.
pub trait IdentityScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityScoreAdjuster {
    pub percent: f64,
}

impl IdentityScoreAdjuster for PercentIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityScoreAdjuster for FlaggedIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityScoreAdjuster {
    pub step: f64,
}

impl IdentityScoreAdjuster for RoundingIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentIdentityScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityScoreAdjuster for SegmentIdentityScoreAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &IdentityScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityScoreAdjuster for TieredIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityScoreAdjuster {
    pub cap: f64,
}

impl IdentityScoreAdjuster for CapIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the identity score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityScoreEngine {
    pub steps: Vec<Rc<dyn IdentityScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityScoreEngine {
    pub fn compute(&mut self, item: &IdentityScore) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_score_engine(flags: &flags::Registry) -> IdentityScoreEngine {
    IdentityScoreEngine {
        steps: vec![
            Rc::new(PercentIdentityScoreAdjuster { percent: 19.5 }) as Rc<dyn IdentityScoreAdjuster>,
            Rc::new(FlaggedIdentityScoreAdjuster {
                flags: flags.clone(),
                flag: "identity-score-global".to_string(),
                percent: 2.1,
            }),
            Rc::new(RoundingIdentityScoreAdjuster { step: 0.5 }),
            Rc::new(SegmentIdentityScoreAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.35),
                    ("central".to_string(), 0.74),
                    ("north".to_string(), 1.21),
                ]),
            }),
            Rc::new(TieredIdentityScoreAdjuster {
                bounds: vec![4285.0, 4360.0],
                rates: vec![0.9, 1.11],
            }),
            Rc::new(CapIdentityScoreAdjuster { cap: 8081.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 75962.7,
    }
}
