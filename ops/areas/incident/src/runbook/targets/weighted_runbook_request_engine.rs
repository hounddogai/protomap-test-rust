use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted runbook request calculation.
pub trait WeightedRunbookRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRunbookRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceWeightedRunbookRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedRunbookRequestAdjuster for SourceWeightedRunbookRequestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &WeightedRunbookRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedRunbookRequestAdjuster {
    pub minimum: f64,
}

impl WeightedRunbookRequestAdjuster for MinimumWeightedRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedRunbookRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRunbookRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRunbookRequestAdjuster for FlaggedWeightedRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRunbookRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted runbook request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRunbookRequestEngine {
    pub steps: Vec<Rc<dyn WeightedRunbookRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRunbookRequestEngine {
    pub fn evaluate(&mut self, item: &WeightedRunbookRequest) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedRunbookRequestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRunbookRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_runbook_request_engine(flags: &flags::Registry) -> WeightedRunbookRequestEngine {
    WeightedRunbookRequestEngine {
        steps: vec![
            Rc::new(SourceWeightedRunbookRequestAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.28), ("south".to_string(), 1.03)]),
            }) as Rc<dyn WeightedRunbookRequestAdjuster>,
            Rc::new(MinimumWeightedRunbookRequestAdjuster { minimum: 4.4 }),
            Rc::new(FlaggedWeightedRunbookRequestAdjuster {
                flags: flags.clone(),
                flag: "weighted-runbook-request-tiered".to_string(),
                percent: -4.5,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 82689.8,
    }
}
