use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rolling segment agreement calculation.
pub trait RollingSegmentAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingSegmentAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingSegmentAgreementAdjuster {
    pub percent: f64,
}

impl RollingSegmentAgreementAdjuster for PercentRollingSegmentAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingSegmentAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingSegmentAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingSegmentAgreementAdjuster for FlaggedRollingSegmentAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingSegmentAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentAgreementWeightOffsetAdjuster {
    pub weight: f64,
}

impl RollingSegmentAgreementAdjuster for RollingSegmentAgreementWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &RollingSegmentAgreement, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingSegmentAgreementAdjuster {
    pub cap: f64,
}

impl RollingSegmentAgreementAdjuster for CapRollingSegmentAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingSegmentAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling segment agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingSegmentAgreementEngine {
    pub steps: Vec<Rc<dyn RollingSegmentAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingSegmentAgreementEngine {
    pub fn price(&mut self, item: &RollingSegmentAgreement) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingSegmentAgreementAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingSegmentAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_segment_agreement_engine(flags: &flags::Registry) -> RollingSegmentAgreementEngine {
    RollingSegmentAgreementEngine {
        steps: vec![
            Rc::new(PercentRollingSegmentAgreementAdjuster { percent: 11.0 })
                as Rc<dyn RollingSegmentAgreementAdjuster>,
            Rc::new(FlaggedRollingSegmentAgreementAdjuster {
                flags: flags.clone(),
                flag: "rolling-segment-agreement-pending".to_string(),
                percent: 6.0,
            }),
            Rc::new(RollingSegmentAgreementWeightOffsetAdjuster { weight: 0.06 }),
            Rc::new(CapRollingSegmentAgreementAdjuster { cap: 1244.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 44154.1,
    }
}
