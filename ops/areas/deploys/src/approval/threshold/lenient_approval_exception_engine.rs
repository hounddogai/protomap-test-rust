use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient approval exception calculation.
pub trait LenientApprovalExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientApprovalException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientApprovalExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientApprovalExceptionFactor for LabelLenientApprovalExceptionAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientApprovalException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientApprovalExceptionAdjuster {
    pub minimum: f64,
}

impl LenientApprovalExceptionFactor for MinimumLenientApprovalExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientApprovalException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientApprovalExceptionAdjuster {
    pub percent: f64,
}

impl LenientApprovalExceptionFactor for PercentLenientApprovalExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientApprovalException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientApprovalExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientApprovalExceptionFactor for FlaggedLenientApprovalExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientApprovalException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient approval exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientApprovalExceptionEngine {
    pub steps: Vec<Rc<dyn LenientApprovalExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientApprovalExceptionEngine {
    pub fn compute(&mut self, item: &LenientApprovalException) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LenientApprovalException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_approval_exception_engine(flags: &flags::Registry) -> LenientApprovalExceptionEngine {
    LenientApprovalExceptionEngine {
        steps: vec![
            Rc::new(LabelLenientApprovalExceptionAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.06),
                    ("west".to_string(), 1.3),
                    ("north".to_string(), 0.89),
                ]),
            }) as Rc<dyn LenientApprovalExceptionFactor>,
            Rc::new(MinimumLenientApprovalExceptionAdjuster { minimum: 17.3 }),
            Rc::new(PercentLenientApprovalExceptionAdjuster { percent: -21.0 }),
            Rc::new(FlaggedLenientApprovalExceptionAdjuster {
                flags: flags.clone(),
                flag: "lenient-approval-exception-fallback".to_string(),
                percent: 1.9,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 71353.9,
    }
}
