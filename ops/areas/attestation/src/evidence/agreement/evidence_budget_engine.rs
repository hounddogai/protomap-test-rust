use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence budget calculation.
pub trait EvidenceBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEvidenceBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceBudgetStep for LabelEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EvidenceBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceBudgetAdjuster {
    pub percent: f64,
}

impl EvidenceBudgetStep for PercentEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceBudgetStep for FlaggedEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceBudgetAdjuster {
    pub step: f64,
}

impl EvidenceBudgetStep for RoundingEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceBudgetAdjuster {
    pub cap: f64,
}

impl EvidenceBudgetStep for CapEvidenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the evidence budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceBudgetEngine {
    pub steps: Vec<Rc<dyn EvidenceBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceBudgetEngine {
    pub fn price(&mut self, item: &EvidenceBudget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_evidence_budget_engine(flags: &flags::Registry) -> EvidenceBudgetEngine {
    EvidenceBudgetEngine {
        steps: vec![
            Rc::new(LabelEvidenceBudgetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.82), ("retail".to_string(), 0.95)]),
            }) as Rc<dyn EvidenceBudgetStep>,
            Rc::new(PercentEvidenceBudgetAdjuster { percent: 14.5 }),
            Rc::new(FlaggedEvidenceBudgetAdjuster {
                flags: flags.clone(),
                flag: "evidence-budget-regional".to_string(),
                percent: -9.7,
            }),
            Rc::new(RoundingEvidenceBudgetAdjuster { step: 5.0 }),
            Rc::new(CapEvidenceBudgetAdjuster { cap: 25548.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 56158.3,
    }
}
