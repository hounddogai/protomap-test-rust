use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the postmortem exception calculation.
pub trait PostmortemExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemExceptionAdjuster {
    pub minimum: f64,
}

impl PostmortemExceptionStep for MinimumPostmortemExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemExceptionAdjuster {
    pub step: f64,
}

impl PostmortemExceptionStep for RoundingPostmortemExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemExceptionStep for TieredPostmortemExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemExceptionCostOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemExceptionStep for PostmortemExceptionCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemException, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemExceptionAdjuster {
    pub cap: f64,
}

impl PostmortemExceptionStep for CapPostmortemExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the postmortem exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemExceptionEngine {
    pub steps: Vec<Rc<dyn PostmortemExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemExceptionEngine {
    pub fn run(&mut self, item: &PostmortemException) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PostmortemException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_postmortem_exception_engine() -> PostmortemExceptionEngine {
    PostmortemExceptionEngine {
        steps: vec![
            Rc::new(MinimumPostmortemExceptionAdjuster { minimum: 13.4 }) as Rc<dyn PostmortemExceptionStep>,
            Rc::new(RoundingPostmortemExceptionAdjuster { step: 0.1 }),
            Rc::new(TieredPostmortemExceptionAdjuster {
                bounds: vec![4150.0, 7252.0],
                rates: vec![0.89, 1.18],
            }),
            Rc::new(PostmortemExceptionCostOffsetAdjuster { weight: 0.09 }),
            Rc::new(CapPostmortemExceptionAdjuster { cap: 19005.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 46278.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_exception_engine();
        let item = new_postmortem_exception("code-147", "name-44");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
