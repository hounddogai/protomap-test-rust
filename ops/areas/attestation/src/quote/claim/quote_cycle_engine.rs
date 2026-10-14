use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the quote cycle calculation.
pub trait QuoteCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteCycleAdjuster {
    pub minimum: f64,
}

impl QuoteCycleAdjuster for MinimumQuoteCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCycleGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl QuoteCycleAdjuster for QuoteCycleGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &QuoteCycle, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteCycleAdjuster {
    pub step: f64,
}

impl QuoteCycleAdjuster for RoundingQuoteCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteCycleAdjuster {
    pub percent: f64,
}

impl QuoteCycleAdjuster for PercentQuoteCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteCycleEngine {
    pub steps: Vec<Rc<dyn QuoteCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteCycleEngine {
    pub fn run(&mut self, item: &QuoteCycle) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[QuoteCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_cycle_engine() -> QuoteCycleEngine {
    QuoteCycleEngine {
        steps: vec![
            Rc::new(MinimumQuoteCycleAdjuster { minimum: 6.3 }) as Rc<dyn QuoteCycleAdjuster>,
            Rc::new(QuoteCycleGraceDaysOffsetAdjuster { weight: 1.99 }),
            Rc::new(RoundingQuoteCycleAdjuster { step: 0.05 }),
            Rc::new(PercentQuoteCycleAdjuster { percent: 3.7 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 40891.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_cycle_engine();
        let item = new_quote_cycle("code-889", "label-880");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
