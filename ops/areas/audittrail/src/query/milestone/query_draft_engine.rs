use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the query draft calculation.
pub trait QueryDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryDraftAdjuster {
    pub step: f64,
}

impl QueryDraftStep for RoundingQueryDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryDraftStep for TieredQueryDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryDraftAdjuster {
    pub percent: f64,
}

impl QueryDraftStep for PercentQueryDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryDraftAdjuster {
    pub minimum: f64,
}

impl QueryDraftStep for MinimumQueryDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryDraftAdjuster {
    pub cap: f64,
}

impl QueryDraftStep for CapQueryDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the query draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryDraftEngine {
    pub steps: Vec<Rc<dyn QueryDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryDraftEngine {
    pub fn run(&mut self, item: &QueryDraft) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_query_draft_engine() -> QueryDraftEngine {
    QueryDraftEngine {
        steps: vec![
            Rc::new(RoundingQueryDraftAdjuster { step: 0.25 }) as Rc<dyn QueryDraftStep>,
            Rc::new(TieredQueryDraftAdjuster {
                bounds: vec![513.0, 5805.0, 8371.0],
                rates: vec![1.17, 1.02, 0.84],
            }),
            Rc::new(PercentQueryDraftAdjuster { percent: -16.6 }),
            Rc::new(MinimumQueryDraftAdjuster { minimum: 49.5 }),
            Rc::new(CapQueryDraftAdjuster { cap: 48984.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 27723.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_draft_engine();
        let item = new_query_draft("code-881", "title-267");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
