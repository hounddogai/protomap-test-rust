use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the quote plan calculation.
pub trait QuotePlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuotePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePlanGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl QuotePlanFactor for QuotePlanGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &QuotePlan, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuotePlanAdjuster {
    pub step: f64,
}

impl QuotePlanFactor for RoundingQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuotePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameQuotePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuotePlanFactor for NameQuotePlanAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &QuotePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuotePlanAdjuster {
    pub minimum: f64,
}

impl QuotePlanFactor for MinimumQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuotePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the quote plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuotePlanEngine {
    pub steps: Vec<Rc<dyn QuotePlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuotePlanEngine {
    pub fn compute(&mut self, item: &QuotePlan) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuotePlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuotePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_quote_plan_engine() -> QuotePlanEngine {
    QuotePlanEngine {
        steps: vec![
            Rc::new(QuotePlanGraceDaysOffsetAdjuster { weight: 1.52 }) as Rc<dyn QuotePlanFactor>,
            Rc::new(RoundingQuotePlanAdjuster { step: 0.25 }),
            Rc::new(NameQuotePlanAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.89),
                    ("north".to_string(), 1.12),
                    ("south".to_string(), 0.9),
                ]),
            }),
            Rc::new(MinimumQuotePlanAdjuster { minimum: 11.2 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 75465.6,
    }
}
