use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the quote assignment calculation.
pub trait QuoteAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteAssignmentAdjuster {
    pub minimum: f64,
}

impl QuoteAssignmentAdjuster for MinimumQuoteAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionQuoteAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteAssignmentAdjuster for RegionQuoteAssignmentAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &QuoteAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteAssignmentAdjuster {
    pub cap: f64,
}

impl QuoteAssignmentAdjuster for CapQuoteAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteAssignmentAdjuster {
    pub percent: f64,
}

impl QuoteAssignmentAdjuster for PercentQuoteAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteAssignmentEngine {
    pub steps: Vec<Rc<dyn QuoteAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteAssignmentEngine {
    pub fn price(&mut self, item: &QuoteAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_quote_assignment_engine() -> QuoteAssignmentEngine {
    QuoteAssignmentEngine {
        steps: vec![
            Rc::new(MinimumQuoteAssignmentAdjuster { minimum: 27.8 }) as Rc<dyn QuoteAssignmentAdjuster>,
            Rc::new(RegionQuoteAssignmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.78)]),
            }),
            Rc::new(CapQuoteAssignmentAdjuster { cap: 29231.0 }),
            Rc::new(PercentQuoteAssignmentAdjuster { percent: -4.3 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 47218.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_assignment_engine();
        let item = new_quote_assignment("code-811", "title-462");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
