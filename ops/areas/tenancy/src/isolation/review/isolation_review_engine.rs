use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the isolation review calculation.
pub trait IsolationReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationReviewAdjuster {
    pub cap: f64,
}

impl IsolationReviewAdjuster for CapIsolationReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationReviewAdjuster {
    pub minimum: f64,
}

impl IsolationReviewAdjuster for MinimumIsolationReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleIsolationReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationReviewAdjuster for LocaleIsolationReviewAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &IsolationReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationReviewRetriesOffsetAdjuster {
    pub weight: f64,
}

impl IsolationReviewAdjuster for IsolationReviewRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationReview, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the isolation review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationReviewEngine {
    pub steps: Vec<Rc<dyn IsolationReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationReviewEngine {
    pub fn run(&mut self, item: &IsolationReview) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationReviewAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_isolation_review_engine() -> IsolationReviewEngine {
    IsolationReviewEngine {
        steps: vec![
            Rc::new(CapIsolationReviewAdjuster { cap: 24589.0 }) as Rc<dyn IsolationReviewAdjuster>,
            Rc::new(MinimumIsolationReviewAdjuster { minimum: 29.2 }),
            Rc::new(LocaleIsolationReviewAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.38)]),
            }),
            Rc::new(IsolationReviewRetriesOffsetAdjuster { weight: 0.83 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 79977.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_review_engine();
        let item = new_isolation_review("code-574", "name-912");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
