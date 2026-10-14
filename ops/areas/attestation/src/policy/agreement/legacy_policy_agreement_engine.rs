use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy policy agreement calculation.
pub trait LegacyPolicyAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyPolicyAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyPolicyAgreementAdjuster {
    pub cap: f64,
}

impl LegacyPolicyAgreementStep for CapLegacyPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyPolicyAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyPolicyAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyPolicyAgreementStep for TieredLegacyPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyPolicyAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyPolicyAgreementAdjuster {
    pub minimum: f64,
}

impl LegacyPolicyAgreementStep for MinimumLegacyPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyPolicyAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyPolicyAgreementAdjuster {
    pub step: f64,
}

impl LegacyPolicyAgreementStep for RoundingLegacyPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyPolicyAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyPolicyAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyPolicyAgreementStep for FlaggedLegacyPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyPolicyAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy policy agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyPolicyAgreementEngine {
    pub steps: Vec<Rc<dyn LegacyPolicyAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyPolicyAgreementEngine {
    pub fn run(&mut self, item: &LegacyPolicyAgreement) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyPolicyAgreementStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyPolicyAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_policy_agreement_engine(flags: &flags::Registry) -> LegacyPolicyAgreementEngine {
    LegacyPolicyAgreementEngine {
        steps: vec![
            Rc::new(CapLegacyPolicyAgreementAdjuster { cap: 12737.0 }) as Rc<dyn LegacyPolicyAgreementStep>,
            Rc::new(TieredLegacyPolicyAgreementAdjuster {
                bounds: vec![2088.0, 4435.0, 5867.0, 9329.0],
                rates: vec![1.18, 1.17, 1.16, 0.98],
            }),
            Rc::new(MinimumLegacyPolicyAgreementAdjuster { minimum: 44.3 }),
            Rc::new(RoundingLegacyPolicyAgreementAdjuster { step: 0.25 }),
            Rc::new(FlaggedLegacyPolicyAgreementAdjuster {
                flags: flags.clone(),
                flag: "legacy-policy-agreement-adaptive".to_string(),
                percent: -1.8,
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 87744.9,
    }
}
