use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity summary calculation.
pub trait IdentitySummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentitySummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentitySummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentitySummaryFactor for FlaggedIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentitySummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentitySummaryAdjuster {
    pub percent: f64,
}

impl IdentitySummaryFactor for PercentIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentitySummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySummaryRateOffsetAdjuster {
    pub weight: f64,
}

impl IdentitySummaryFactor for IdentitySummaryRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &IdentitySummary, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelIdentitySummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentitySummaryFactor for LabelIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &IdentitySummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentitySummaryAdjuster {
    pub step: f64,
}

impl IdentitySummaryFactor for RoundingIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentitySummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the identity summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentitySummaryEngine {
    pub steps: Vec<Rc<dyn IdentitySummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentitySummaryEngine {
    pub fn compute(&mut self, item: &IdentitySummary) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentitySummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentitySummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_summary_engine(flags: &flags::Registry) -> IdentitySummaryEngine {
    IdentitySummaryEngine {
        steps: vec![
            Rc::new(FlaggedIdentitySummaryAdjuster {
                flags: flags.clone(),
                flag: "identity-summary-adaptive".to_string(),
                percent: -8.4,
            }) as Rc<dyn IdentitySummaryFactor>,
            Rc::new(PercentIdentitySummaryAdjuster { percent: -0.3 }),
            Rc::new(IdentitySummaryRateOffsetAdjuster { weight: 1.56 }),
            Rc::new(LabelIdentitySummaryAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.99)]),
            }),
            Rc::new(RoundingIdentitySummaryAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 92082.0,
    }
}
