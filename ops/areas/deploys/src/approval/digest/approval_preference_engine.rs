use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval preference calculation.
pub trait ApprovalPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalPreferenceStep for TieredApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPreferenceBurstOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalPreferenceStep for ApprovalPreferenceBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalPreference, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalPreferenceAdjuster {
    pub step: f64,
}

impl ApprovalPreferenceStep for RoundingApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelApprovalPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalPreferenceStep for LabelApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ApprovalPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalPreferenceStep for FlaggedApprovalPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalPreferenceEngine {
    pub steps: Vec<Rc<dyn ApprovalPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalPreferenceEngine {
    pub fn run(&mut self, item: &ApprovalPreference) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalPreferenceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_preference_engine(flags: &flags::Registry) -> ApprovalPreferenceEngine {
    ApprovalPreferenceEngine {
        steps: vec![
            Rc::new(TieredApprovalPreferenceAdjuster {
                bounds: vec![1413.0, 1949.0, 2721.0, 8683.0],
                rates: vec![1.0, 0.82, 1.02, 0.93],
            }) as Rc<dyn ApprovalPreferenceStep>,
            Rc::new(ApprovalPreferenceBurstOffsetAdjuster { weight: 0.93 }),
            Rc::new(RoundingApprovalPreferenceAdjuster { step: 1.0 }),
            Rc::new(LabelApprovalPreferenceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.14)]),
            }),
            Rc::new(FlaggedApprovalPreferenceAdjuster {
                flags: flags.clone(),
                flag: "approval-preference-fallback".to_string(),
                percent: 4.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 13465.0,
    }
}
