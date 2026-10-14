use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the allowlist agreement calculation.
pub trait AllowlistAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistAgreementAdjuster {
    pub percent: f64,
}

impl AllowlistAgreementStep for PercentAllowlistAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistAgreementAdjuster {
    pub cap: f64,
}

impl AllowlistAgreementStep for CapAllowlistAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAllowlistAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistAgreementStep for NameAllowlistAgreementAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &AllowlistAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAgreementMarginOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistAgreementStep for AllowlistAgreementMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistAgreement, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistAgreementStep for FlaggedAllowlistAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the allowlist agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistAgreementEngine {
    pub steps: Vec<Rc<dyn AllowlistAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistAgreementEngine {
    pub fn price(&mut self, item: &AllowlistAgreement) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistAgreementStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_agreement_engine(flags: &flags::Registry) -> AllowlistAgreementEngine {
    AllowlistAgreementEngine {
        steps: vec![
            Rc::new(PercentAllowlistAgreementAdjuster { percent: 23.2 }) as Rc<dyn AllowlistAgreementStep>,
            Rc::new(CapAllowlistAgreementAdjuster { cap: 40655.0 }),
            Rc::new(NameAllowlistAgreementAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.73), ("central".to_string(), 1.15)]),
            }),
            Rc::new(AllowlistAgreementMarginOffsetAdjuster { weight: 1.55 }),
            Rc::new(FlaggedAllowlistAgreementAdjuster {
                flags: flags.clone(),
                flag: "allowlist-agreement-rolling".to_string(),
                percent: -9.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 24544.9,
    }
}
