use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional verifier summary calculation.
pub trait RegionalVerifierSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalVerifierSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalVerifierSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalVerifierSummaryStep for NameRegionalVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalVerifierSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalVerifierSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalVerifierSummaryStep for FlaggedRegionalVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalVerifierSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalVerifierSummaryAdjuster {
    pub cap: f64,
}

impl RegionalVerifierSummaryStep for CapRegionalVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalVerifierSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalVerifierSummaryAdjuster {
    pub percent: f64,
}

impl RegionalVerifierSummaryStep for PercentRegionalVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalVerifierSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional verifier summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalVerifierSummaryEngine {
    pub steps: Vec<Rc<dyn RegionalVerifierSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalVerifierSummaryEngine {
    pub fn evaluate(&mut self, item: &RegionalVerifierSummary) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionalVerifierSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_verifier_summary_engine(flags: &flags::Registry) -> RegionalVerifierSummaryEngine {
    RegionalVerifierSummaryEngine {
        steps: vec![
            Rc::new(NameRegionalVerifierSummaryAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.02),
                    ("online".to_string(), 1.3),
                    ("retail".to_string(), 1.17),
                ]),
            }) as Rc<dyn RegionalVerifierSummaryStep>,
            Rc::new(FlaggedRegionalVerifierSummaryAdjuster {
                flags: flags.clone(),
                flag: "regional-verifier-summary-primary".to_string(),
                percent: 3.5,
            }),
            Rc::new(CapRegionalVerifierSummaryAdjuster { cap: 13901.0 }),
            Rc::new(PercentRegionalVerifierSummaryAdjuster { percent: 8.2 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 12166.0,
    }
}
