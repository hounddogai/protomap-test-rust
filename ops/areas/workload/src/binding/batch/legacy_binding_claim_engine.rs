use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy binding claim calculation.
pub trait LegacyBindingClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyBindingClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyBindingClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyBindingClaimModifier for TieredLegacyBindingClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyBindingClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingClaimRetriesOffsetAdjuster {
    pub weight: f64,
}

impl LegacyBindingClaimModifier for LegacyBindingClaimRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyBindingClaim, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyBindingClaimAdjuster {
    pub percent: f64,
}

impl LegacyBindingClaimModifier for PercentLegacyBindingClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyBindingClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyBindingClaimAdjuster {
    pub step: f64,
}

impl LegacyBindingClaimModifier for RoundingLegacyBindingClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyBindingClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyBindingClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyBindingClaimModifier for FlaggedLegacyBindingClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyBindingClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyBindingClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyBindingClaimModifier for NameLegacyBindingClaimAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyBindingClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy binding claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyBindingClaimEngine {
    pub steps: Vec<Rc<dyn LegacyBindingClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyBindingClaimEngine {
    pub fn evaluate(&mut self, item: &LegacyBindingClaim) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyBindingClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyBindingClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_binding_claim_engine(flags: &flags::Registry) -> LegacyBindingClaimEngine {
    LegacyBindingClaimEngine {
        steps: vec![
            Rc::new(TieredLegacyBindingClaimAdjuster {
                bounds: vec![7618.0, 9089.0],
                rates: vec![0.84, 0.94],
            }) as Rc<dyn LegacyBindingClaimModifier>,
            Rc::new(LegacyBindingClaimRetriesOffsetAdjuster { weight: 1.64 }),
            Rc::new(PercentLegacyBindingClaimAdjuster { percent: 21.6 }),
            Rc::new(RoundingLegacyBindingClaimAdjuster { step: 1.0 }),
            Rc::new(FlaggedLegacyBindingClaimAdjuster {
                flags: flags.clone(),
                flag: "legacy-binding-claim-fallback".to_string(),
                percent: 0.0,
            }),
            Rc::new(NameLegacyBindingClaimAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.87),
                    ("east".to_string(), 1.3),
                    ("retail".to_string(), 0.85),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 20153.5,
    }
}
